extern crate serde_json;
extern crate indy_crypto;

use std::collections::{HashMap, HashSet};

use self::indy_crypto::cl::*;

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct CredentialDefinition {
    #[serde(rename = "ref")]
    pub schema_seq_no: i32,
    #[serde(rename = "origin")]
    pub issuer_did: String,
    pub signature_type: String,
    pub data: CredentialDefinitionData
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct CredentialDefinitionData {
    pub primary: CredentialPrimaryPublicKey,
    pub revocation: Option<serde_json::Value>,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub enum ResponseType {
    REQNACK,
    REPLY,
    REJECT
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub op: ResponseType,
    pub reason: String,
    pub req_id: u64,
    pub identifier: String
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct Reply<T> {
    pub op: String,
    pub result: T,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNymReplyResult {
    pub identifier: String,
    pub req_id: u64,
    #[serde(rename = "type")]
    pub _type: String,
    pub data: Option<String>,
    pub dest: String
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNymResultData {
    pub identifier: String,
    pub dest: String,
    pub role: Option<String>,
    pub verkey: Option<String>
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAttribReplyResult {
    pub     identifier: String,
    pub   req_id: u64,
    #[serde(rename = "type")]
    pub   _type: String,
    pub   data: Option<String>,
    pub  dest: String,
    pub  seq_no: Option<i32>
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaReplyResult {
    pub identifier: String,
    pub req_id: u64,
    pub seq_no: Option<i32>,
    //For tests/ In normal case seq_no exists
    #[serde(rename = "type")]
    pub  _type: String,
    pub  data: Option<GetSchemaResultData>,
    pub  dest: Option<String>
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct GetSchemaResultData {
    pub attr_names: HashSet<String>,
    pub name: String,
    pub origin: Option<String>,
    pub version: String
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct GetClaimDefReplyResult {
    pub identifier: String,
    #[serde(rename = "reqId")]
    pub req_id: u64,
    #[serde(rename = "seqNo")]
    pub seq_no: i32,
    #[serde(rename = "type")]
    pub _type: String,
    pub data: CredentialDefinitionData,
    pub origin: String,
    pub signature_type: String,
    #[serde(rename = "ref")]
    pub  _ref: i32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxnResult {
    pub identifier: String,
    #[serde(rename = "reqId")]
    pub req_id: u64,
    #[serde(rename = "seqNo")]
    pub seq_no: Option<i32>,
    #[serde(rename = "type")]
    pub _type: String,
    pub data: Option<serde_json::Value>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaResult {
    pub identifier: String,
    #[serde(rename = "reqId")]
    pub req_id: u64,
    #[serde(rename = "seqNo")]
    pub seq_no: i32,
    #[serde(rename = "type")]
    pub _type: String,
    pub data: Option<SchemaData>
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schema {
    #[serde(rename = "seqNo")]
    pub seq_no: i32,
    pub dest: String,
    pub data: SchemaData
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct SchemaKey {
    pub name: String,
    pub version: String,
    pub did: String
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaData {
    pub name: String,
    pub version: String,
    pub attr_names: HashSet<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialOffer {
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub key_correctness_proof: CredentialKeyCorrectnessProof,
    pub nonce: Nonce
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct CredentialOfferInfo {
    pub issuer_did: String,
    pub schema_key: SchemaKey,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialsForProofRequest {
    pub attrs: HashMap<String, Vec<RequestedCredential>>,
    pub predicates: HashMap<String, Vec<RequestedCredential>>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestedCredential {
    pub cred_info: CredentialInfo,
    pub freshness: Option<u64>
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct CredentialInfo {
    pub referent: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Credential {
    pub values: HashMap<String, Vec<String>>,
    pub schema_key: SchemaKey,
    pub signature: CredentialSignature,
    pub signature_correctness_proof: SignatureCorrectnessProof,
    pub issuer_did: String,
    pub rev_reg_seq_no: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FullProof {
    pub proof: Proof,
    pub requested_proof: RequestedProof,
    pub identifiers: HashMap<String, Identifier>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestedProof {
    pub revealed_attrs: HashMap<String, RevealedAttributeInfo>,
    pub unrevealed_attrs: HashMap<String, String>,
    pub self_attested_attrs: HashMap<String, String>,
    pub predicates: HashMap<String, String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevealedAttributeInfo {
    pub referent: String,
    pub raw: String,
    pub encoded: String
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub schema_id: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>
}