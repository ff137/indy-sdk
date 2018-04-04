extern crate indy_crypto;

use self::indy_crypto::cl::{
    CredentialSignature,
    RevocationRegistry,
    SignatureCorrectnessProof,
    Witness
};
use self::indy_crypto::utils::json::{JsonDecodable, JsonEncodable};

use super::filter::Filtering;

use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Credential {
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub values: HashMap<String, AttributeValues>,
    pub signature: CredentialSignature,
    pub signature_correctness_proof: SignatureCorrectnessProof,
    pub rev_reg: Option<RevocationRegistry>,
    pub witness: Option<Witness>
}

impl Credential {
    pub fn schema_id(&self) -> String {
        let parts: Vec<&str> = self.cred_def_id.split_terminator(":").collect::<Vec<&str>>(); // TODO: FIXME
        if parts.len() == 4 {
            parts[3].to_string()
        } else {
            parts[3..].join(":").to_string()
        }
    }
}

impl JsonEncodable for Credential {}

impl<'a> JsonDecodable<'a> for Credential {}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct CredentialInfo {
    pub referent: String,
    pub attrs: HashMap<String, String>,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub cred_rev_id: Option<String>
}

impl CredentialInfo {
    fn parts(&self) -> Vec<&str> {
        self.cred_def_id.split_terminator(":").collect::<Vec<&str>>()
    }
}

impl Filtering for CredentialInfo {
    fn schema_id(&self) -> String {
        if self.parts().len() == 4 {
            self.parts()[3].to_string()
        } else {
            self.parts()[3..].join(":").to_string()
        }
    }
    fn schema_issuer_did(&self) -> String { self.parts().get(3).map(|s|s.to_string()).unwrap_or(String::new()) }
    fn schema_name(&self) -> String { self.parts().get(5).map(|s|s.to_string()).unwrap_or(String::new()) }
    fn schema_version(&self) -> String { self.parts().get(6).map(|s|s.to_string()).unwrap_or(String::new()) }
    fn issuer_did(&self) -> String { self.parts()[0].to_string() }
    fn cred_def_id(&self) -> String { self.cred_def_id.to_string() }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AttributeValues {
    pub raw: String,
    pub encoded: String
}