extern crate indy_crypto;
extern crate serde;
extern crate serde_json;

use self::serde::ser::{self, Serialize, Serializer};

use self::indy_crypto::utils::json::{JsonDecodable, JsonEncodable};
use self::indy_crypto::cl::RevocationKeyPublic;

use super::build_id;

use std::collections::HashMap;

pub const CL_ACCUM: &'static str = "CL_ACCUM";
pub const REV_REG_DEG_MARKER: &'static str = "\x04";

#[derive(Deserialize, Debug, Serialize)]
pub struct RevocationRegistryConfig {
    pub issuance_type: Option<String>,
    pub max_cred_num: Option<u32>
}

impl<'a> JsonDecodable<'a> for RevocationRegistryConfig {}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Serialize, PartialEq, Clone)]
pub enum IssuanceType {
    ISSUANCE_BY_DEFAULT,
    ISSUANCE_ON_DEMAND
}

impl IssuanceType {
    pub fn to_bool(&self) -> bool {
        self.clone() == IssuanceType::ISSUANCE_BY_DEFAULT
    }
}

impl<'a> JsonDecodable<'a> for IssuanceType {}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub enum RegistryType {
    CL_ACCUM,
}

impl<'a> JsonDecodable<'a> for RegistryType {}

impl RegistryType {
    pub fn to_str(&self) -> &'static str {
        match self {
            &RegistryType::CL_ACCUM => CL_ACCUM
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionValue {
    pub issuance_type: IssuanceType,
    pub max_cred_num: u32,
    pub public_keys: RevocationRegistryDefinitionValuePublicKeys,
    pub tails_hash: String,
    pub tails_location: String
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionValuePublicKeys {
    pub accum_key: RevocationKeyPublic
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionV1 {
    pub id: String,
    #[serde(rename = "revocDefType")]
    pub type_: RegistryType,
    pub tag: String,
    pub cred_def_id: String,
    pub value: RevocationRegistryDefinitionValue
}

impl RevocationRegistryDefinitionV1 {
    const VERSION: &'static str = "1";
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RevocationRegistryDefinition {
    RevocationRegistryDefinitionV1(RevocationRegistryDefinitionV1)
}

impl RevocationRegistryDefinition {
    pub fn rev_reg_id(did: &str, cred_def_id: &str, rev_reg_type: &RegistryType, tag: &str) -> String {
        build_id(did, REV_REG_DEG_MARKER, Some(cred_def_id), rev_reg_type.to_str(), tag)
    }
}

impl JsonEncodable for RevocationRegistryDefinition {}

impl<'a> JsonDecodable<'a> for RevocationRegistryDefinition {}

impl Serialize for RevocationRegistryDefinition
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match *self {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(ref rev_reg_def) => {
                let mut v = serde_json::to_value(rev_reg_def).map_err(ser::Error::custom)?;
                v["ver"] = serde_json::Value::String(RevocationRegistryDefinitionV1::VERSION.to_string());
                v.serialize(serializer)
            }
        }
    }
}

impl From<RevocationRegistryDefinition> for RevocationRegistryDefinitionV1 {
    fn from(rev_reg_def: RevocationRegistryDefinition) -> Self {
        match rev_reg_def {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_reg_def) => rev_reg_def
        }
    }
}

pub fn rev_reg_defs_map_to_rev_reg_defs_v1_map(rev_reg_defs: HashMap<String, RevocationRegistryDefinition>) -> HashMap<String, RevocationRegistryDefinitionV1> {
    let mut rev_reg_defs_v1: HashMap<String, RevocationRegistryDefinitionV1> = HashMap::new();

    for (rev_reg_id, rev_reg_def) in rev_reg_defs {
        rev_reg_defs_v1.insert(rev_reg_id, RevocationRegistryDefinitionV1::from(rev_reg_def));
    }

    rev_reg_defs_v1
}
