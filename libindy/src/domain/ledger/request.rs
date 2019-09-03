use serde;
use serde_json;
use time;

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use domain::crypto::did::DidValue;

pub const DEFAULT_LIBIDY_DID: &str = "LibindyDid111111111111";

pub struct ProtocolVersion {}

lazy_static! {
    pub static ref PROTOCOL_VERSION: AtomicUsize = AtomicUsize::new(1);
}

impl ProtocolVersion {
    pub fn set(version: usize) {
        PROTOCOL_VERSION.store(version, Ordering::Relaxed);
    }

    pub fn get() -> usize {
        PROTOCOL_VERSION.load(Ordering::Relaxed)
    }

    pub fn is_node_1_3() -> bool {
        ProtocolVersion::get() == 1
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxnAuthrAgrmtAcceptanceData {
    pub mechanism: String,
    pub taa_digest: String,
    pub time: u64
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request<T: serde::Serialize> {
    pub req_id: u64,
    pub identifier: DidValue,
    pub operation: T,
    pub protocol_version: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signatures: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taa_acceptance: Option<TxnAuthrAgrmtAcceptanceData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endorser: Option<String>
}

impl<T: serde::Serialize> Request<T> {
    pub fn new(req_id: u64, identifier: DidValue, operation: T, protocol_version: usize) -> Request<T> {
        Request {
            req_id,
            identifier,
            operation,
            protocol_version: Some(protocol_version),
            signature: None,
            signatures: None,
            taa_acceptance: None,
            endorser: None,
        }
    }

    pub fn build_request(identifier: Option<&DidValue>, operation: T) -> Result<String, serde_json::Error> {
        let req_id = time::get_time().sec as u64 * (1e9 as u64) + time::get_time().nsec as u64;
        let identifier = identifier.unwrap_or(&DidValue(DEFAULT_LIBIDY_DID.to_string()));
        serde_json::to_string(&Request::new(req_id, identifier.clone(), operation, ProtocolVersion::get()))
    }
}