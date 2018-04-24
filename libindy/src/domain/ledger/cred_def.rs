extern crate serde;
extern crate serde_json;
extern crate indy_crypto;

use super::constants::{CRED_DEF, GET_CRED_DEF};

use self::indy_crypto::utils::json::{JsonDecodable, JsonEncodable};

use super::super::anoncreds::credential_definition::{CredentialDefinitionData, CredentialDefinitionV1, SignatureType};

#[derive(Serialize, Debug)]
pub struct CredDefOperation {
    #[serde(rename = "ref")]
    pub _ref: i32,
    pub data: CredentialDefinitionData,
    #[serde(rename = "type")]
    pub _type: String,
    pub signature_type: String
}

impl CredDefOperation {
    pub fn new(data: CredentialDefinitionV1) -> CredDefOperation {
        CredDefOperation {
            _ref: data.schema_id.parse::<i32>().unwrap_or(0),
            // TODO: FIXME
            signature_type: data.signature_type.to_str().to_string(),
            data: data.value,
            _type: CRED_DEF.to_string()
        }
    }
}

impl JsonEncodable for CredDefOperation {}

#[derive(Serialize, PartialEq, Debug)]
pub struct GetCredDefOperation {
    #[serde(rename = "type")]
    pub _type: String,
    #[serde(rename = "ref")]
    pub _ref: i32,
    pub signature_type: String,
    pub origin: String
}

impl GetCredDefOperation {
    pub fn new(_ref: i32, signature_type: String, origin: String) -> GetCredDefOperation {
        GetCredDefOperation {
            _type: GET_CRED_DEF.to_string(),
            _ref,
            signature_type,
            origin
        }
    }
}

impl JsonEncodable for GetCredDefOperation {}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetCredDefReplyResult {
    pub identifier: String,
    #[serde(rename = "reqId")]
    pub req_id: u64,
    #[serde(rename = "ref")]
    pub ref_: u64,
    #[serde(rename = "seqNo")]
    pub seq_no: i32,
    #[serde(rename = "type")]
    pub  _type: String,
    pub  signature_type: SignatureType,
    pub  origin: String,
    pub  data: CredentialDefinitionData
}

impl<'a> JsonDecodable<'a> for GetCredDefReplyResult {}