extern crate rusoto;
extern crate rust_apex;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::default::Default;
use std::error::Error;
use std::fmt::{Display, Formatter, Error as FmtError};
use std::io::Write;

use rusoto::default_tls_client;
use rusoto::{DefaultCredentialsProvider, Region};
use rusoto::s3::{GetObjectError, GetObjectRequest, S3Client};
use serde_json::{Value, to_value};


#[derive(Debug)]
enum SignXPIError {
    S3GetObjectError(GetObjectError),
    S3GetObjectHasNoBody,
}

impl Display for SignXPIError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "{:?}", self)
    }
}

impl Error for SignXPIError {
    fn description(&self) -> &str {
        "dummy"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct S3BucketInfo {
    name: String,
    arn: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct S3ObjectInfo {
    key: String,
    size: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct S3Path {
    bucket: S3BucketInfo,
    object: S3ObjectInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct S3Event {
    event_time: String,
    event_name: String,  // FIXME: enum
    aws_region: String,
    s3: S3Path,
}

#[derive(Debug, Serialize, Deserialize)]
struct S3BatchEvent {
    #[serde(rename = "Records")]
    records: Vec<S3Event>,
}

#[derive(Debug, Serialize, Deserialize)]
enum S3EventResponse {
    SkipFileNotXPI(String),
    SkipEventNotObjectCreated(String),
    UploadedXPI(S3Path),
}

#[derive(Debug, Serialize, Deserialize)]
struct S3BatchResponse {
    responses: Vec<S3EventResponse>
}

fn main() {
    let provider = DefaultCredentialsProvider::new().unwrap();
    let client = S3Client::new(default_tls_client().unwrap(), provider, Region::UsWest2);
    rust_apex::run::<_, _, SignXPIError, _>(|input: S3BatchEvent, c: rust_apex::Context| {
        let responses = input.records.iter().map(|event| {
            let ref filename = event.s3.object.key;
            if !filename.ends_with(".xpi") {
                return Ok(S3EventResponse::SkipFileNotXPI(filename.clone()));
            }

            if !event.event_name.starts_with("ObjectCreated") {
                return Ok(S3EventResponse::SkipEventNotObjectCreated(event.event_name.clone()));
            }

            let mut get_object_request = GetObjectRequest {
                bucket: event.s3.bucket.name.to_owned(),
                key: filename.to_owned(),
                response_content_type: Some("application/octet-stream".to_owned()),
                ..Default::default()
            };
            let response = try!(client.get_object(&get_object_request).map_err(SignXPIError::S3GetObjectError));
            let mut stderr = std::io::stderr();
            writeln!(&mut stderr, "Got response: {:?}", &response);
            writeln!(&mut stderr, "Body length: {:?}", try!(response.body.ok_or(SignXPIError::S3GetObjectHasNoBody)).len());

            // FIXME: point to some other XPI
            Ok(S3EventResponse::UploadedXPI(event.s3.clone()))
        }).collect();
        Ok(S3BatchResponse { responses: try!(responses) })
    });
}
