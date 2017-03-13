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

#[derive(Debug, Serialize, Deserialize)]
struct S3BucketInfo {
    name: String,
    arn: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct S3ObjectInfo {
    key: String,
    size: i32,
}

#[derive(Debug, Serialize, Deserialize)]
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

fn main() {
    rust_apex::run::<_, _, DummyError, _>(|input: S3BatchEvent, c: rust_apex::Context| {
        let mut bt = BTreeMap::new();
        bt.insert("c", to_value(&c).unwrap());
        bt.insert("i", to_value(&input).unwrap());
        Ok(bt)
    });
}
