extern crate rust_apex;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::error::Error;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter, Error as FmtError};

use serde_json::{Value, to_value};

#[derive(Debug)]
struct DummyError;

impl Display for DummyError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "{:?}", self)
    }
}

impl Error for DummyError {
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
    rust_apex::run::<_, _, DummyError, _>(|input: S3BatchEvent, c: rust_apex::Context| {
        let responses = input.records.iter().map(|event| {
            let ref filename = event.s3.object.key;
            if !filename.ends_with(".xpi") {
                return S3EventResponse::SkipFileNotXPI(filename.clone());
            }

            if !event.event_name.starts_with("ObjectCreated") {
                return S3EventResponse::SkipEventNotObjectCreated(event.event_name.clone());
            }

            // FIXME: point to some other XPI
            S3EventResponse::UploadedXPI(event.s3.clone())
        }).collect();
        Ok(S3BatchResponse { responses: responses })
    });
}
