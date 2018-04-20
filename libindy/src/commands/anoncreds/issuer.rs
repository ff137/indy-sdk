extern crate serde_json;
extern crate indy_crypto;

use errors::indy::IndyError;
use errors::wallet::WalletError;
use errors::anoncreds::AnoncredsError;
use errors::common::CommonError;

use services::anoncreds::AnoncredsService;
use services::anoncreds::helpers::parse_cred_rev_id;
use services::blob_storage::BlobStorageService;
use services::pool::PoolService;
use services::wallet::{WalletService, WalletRecordRetrieveOptions};
use services::crypto::CryptoService;
use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use self::indy_crypto::cl::{
    CredentialKeyCorrectnessProof,
    CredentialPrivateKey,
    RevocationKeyPrivate,
    RevocationRegistryDelta as CryptoRevocationRegistryDelta,
    Witness,
    new_nonce
};
use self::indy_crypto::utils::json::{JsonDecodable, JsonEncodable};
use super::tails::{SDKTailsAccessor, store_tails_from_generator};
use domain::anoncreds::schema::{Schema, SchemaV1};
use domain::anoncreds::credential_definition::{
    CredentialDefinition,
    CredentialDefinitionV1,
    CredentialDefinitionConfig,
    SignatureType
};
use domain::anoncreds::revocation_registry_definition::{
    RevocationRegistryConfig,
    IssuanceType,
    RegistryType,
    RevocationRegistryDefinitionValue,
    RevocationRegistryDefinition,
    RevocationRegistryDefinitionV1
};
use domain::anoncreds::revocation_registry::{
    RevocationRegistry,
    RevocationRegistryV1
};
use domain::anoncreds::revocation_registry_delta::{
    RevocationRegistryDelta,
    RevocationRegistryDeltaV1
};
use domain::anoncreds::credential::{AttributeValues, Credential};
use domain::anoncreds::credential_offer::CredentialOffer;
use domain::anoncreds::credential_request::CredentialRequest;

pub enum IssuerCommand {
    CreateSchema(
        String, // issuer did
        String, // name
        String, // version
        String, // attribute names
        Box<Fn(Result<(String, String), IndyError>) + Send>),
    CreateAndStoreCredentialDefinition(
        i32, // wallet handle
        String, // issuer did
        String, // schema json
        String, // tag
        Option<String>, // type
        String, // config_json
        Box<Fn(Result<(String, String), IndyError>) + Send>),
    CreateAndStoreRevocationRegistry(
        i32, // wallet handle
        String, // issuer did
        Option<String>, // type
        String, // tag
        String, // credential definition id
        String, // config
        i32, // tails writer handle
        Box<Fn(Result<(String, String, String), IndyError>) + Send>),
    CreateCredentialOffer(
        i32, // wallet handle
        String, // credential definition id
        Box<Fn(Result<String, IndyError>) + Send>),
    CreateCredential(
        i32, // wallet handle
        String, // credential offer json
        String, // credential request json
        String, // credential values json
        Option<String>, // revocation registry id
        Option<i32>, // blob storage reader config handle
        Box<Fn(Result<(String, Option<String>, Option<String>), IndyError>) + Send>),
    RevokeCredential(
        i32, // wallet handle
        i32, // blob storage reader config handle
        String, //revocation revoc id
        String, //credential revoc id
        Box<Fn(Result<String, IndyError>) + Send>),
    /*    RecoverCredential(
            i32, // wallet handle
            i32, // blob storage reader config handle
            String, //revocation revoc id
            String, //credential revoc id
            Box<Fn(Result<String, IndyError>) + Send>),*/
    MergeRevocationRegistryDeltas(
        String, //revocation registry delta json
        String, //other revocation registry delta json
        Box<Fn(Result<String, IndyError>) + Send>)
}

pub struct IssuerCommandExecutor {
    pub anoncreds_service: Rc<AnoncredsService>,
    pub blob_storage_service: Rc<BlobStorageService>,
    pub pool_service: Rc<PoolService>,
    pub wallet_service: Rc<WalletService>,
    pub crypto_service: Rc<CryptoService>
}

impl IssuerCommandExecutor {
    pub fn new(anoncreds_service: Rc<AnoncredsService>,
               pool_service: Rc<PoolService>,
               blob_storage_service: Rc<BlobStorageService>,
               wallet_service: Rc<WalletService>,
               crypto_service: Rc<CryptoService>) -> IssuerCommandExecutor {
        IssuerCommandExecutor {
            anoncreds_service,
            pool_service,
            blob_storage_service,
            wallet_service,
            crypto_service,
        }
    }

    pub fn execute(&self, command: IssuerCommand) {
        match command {
            IssuerCommand::CreateSchema(issuer_did, name, version, attrs, cb) => {
                trace!(target: "issuer_command_executor", "CreateSchema command received");
                cb(self.create_schema(&issuer_did, &name, &version, &attrs));
            }
            IssuerCommand::CreateAndStoreCredentialDefinition(wallet_handle, issuer_did, schema_json, tag, type_, config_json, cb) => {
                trace!(target: "issuer_command_executor", "CreateAndStoreCredentialDefinition command received");
                cb(self.create_and_store_credential_definition(wallet_handle, &issuer_did, &schema_json, &tag,
                                                               type_.as_ref().map(String::as_str), &config_json));
            }
            IssuerCommand::CreateAndStoreRevocationRegistry(wallet_handle, issuer_did, type_, tag, cred_def_id, config_json,
                                                            tails_writer_handle, cb) => {
                trace!(target: "issuer_command_executor", "CreateAndStoreRevocationRegistryRegistry command received");
                cb(self.create_and_store_revocation_registry(wallet_handle,
                                                             &issuer_did,
                                                             type_.as_ref().map(String::as_str),
                                                             &tag,
                                                             &cred_def_id,
                                                             &config_json,
                                                             tails_writer_handle));
            }
            IssuerCommand::CreateCredentialOffer(wallet_handle, cred_def_id, cb) => {
                trace!(target: "issuer_command_executor", "CreateCredentialOffer command received");
                cb(self.create_credential_offer(wallet_handle, &cred_def_id));
            }
            IssuerCommand::CreateCredential(wallet_handle, cred_offer_json, cred_req_json, cred_values_json, rev_reg_id, blob_storage_reader_handle, cb) => {
                info!(target: "issuer_command_executor", "CreateCredential command received");
                cb(self.new_credential(wallet_handle, &cred_offer_json, &cred_req_json, &cred_values_json, rev_reg_id.as_ref().map(String::as_str), blob_storage_reader_handle));
            }
            IssuerCommand::RevokeCredential(wallet_handle, blob_storage_reader_handle, rev_reg_id, cred_revoc_id, cb) => {
                trace!(target: "issuer_command_executor", "RevokeCredential command received");
                cb(self.revoke_credential(wallet_handle, blob_storage_reader_handle, &rev_reg_id, &cred_revoc_id));
            }
            /*            IssuerCommand::RecoverCredential(wallet_handle, blob_storage_reader_handle, rev_reg_id, cred_revoc_id, cb) => {
                            trace!(target: "issuer_command_executor", "RecoverCredential command received");
                            cb(self.recovery_credential(wallet_handle, blob_storage_reader_handle, &rev_reg_id, &cred_revoc_id));
                        }*/
            IssuerCommand::MergeRevocationRegistryDeltas(rev_reg_delta_json, other_rev_reg_delta_json, cb) => {
                trace!(target: "issuer_command_executor", "MergeRevocationRegistryDeltas command received");
                cb(self.merge_revocation_registry_deltas(&rev_reg_delta_json, &other_rev_reg_delta_json));
            }
        };
    }

    fn create_schema(&self,
                     issuer_did: &str,
                     name: &str,
                     version: &str,
                     attrs: &str) -> Result<(String, String), IndyError> {
        trace!("create_schema >>> issuer_did: {:?}, name: {:?}, version: {:?}, attrs: {:?}", issuer_did, name, version, attrs);

        self.crypto_service.validate_did(issuer_did)?;

        let attrs: HashSet<String> = serde_json::from_str(attrs)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize AttributeNames: {:?}", err)))?;

        if attrs.is_empty() {
            return Err(IndyError::CommonError(CommonError::InvalidStructure(format!("List of Schema attributes is empty"))));
        }

        let schema_id = Schema::schema_id(issuer_did, name, version);

        let schema = Schema::SchemaV1(SchemaV1 {
            id: schema_id.clone(),
            name: name.to_string(),
            version: version.to_string(),
            attr_names: attrs,
            seq_no: None
        });

        let schema_json = schema.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize Schema: {:?}", err)))?;

        trace!("create_schema <<< schema_id: {:?}, schema_json: {:?}", schema_id, schema_json);

        Ok((schema_id, schema_json))
    }

    fn create_and_store_credential_definition(&self,
                                              wallet_handle: i32,
                                              issuer_did: &str,
                                              schema_json: &str,
                                              tag: &str,
                                              type_: Option<&str>,
                                              config_json: &str) -> Result<(String, String), IndyError> {
        trace!("create_and_store_credential_definition >>> wallet_handle: {:?}, issuer_did: {:?}, schema_json: {:?}, tag: {:?}, \
              type_: {:?}, config_json: {:?}", wallet_handle, issuer_did, schema_json, tag, type_, config_json);

        self.crypto_service.validate_did(issuer_did)?;

        let schema: SchemaV1 = SchemaV1::from(Schema::from_json(schema_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize Schema: {:?}", err)))?);

        let cred_def_config: CredentialDefinitionConfig = CredentialDefinitionConfig::from_json(config_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize CredentialDefinitionConfig: {:?}", err)))?;

        let signature_type = match type_ {
            Some(type_) =>
                SignatureType::from_json(&format!("\"{}\"", type_))
                    .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize SignatureType: {:?}", err)))?,
            None => SignatureType::CL,
        };

        let schema_id = schema.seq_no.map(|n| n.to_string()).unwrap_or(schema.id.clone());

        let cred_def_id = CredentialDefinition::cred_def_id(issuer_did, &schema_id, &signature_type.to_str()); // TODO: FIXME

        if self._wallet_get_cred_def(wallet_handle, &cred_def_id).is_ok() {
            return Err(IndyError::AnoncredsError(AnoncredsError::CredDefAlreadyExists(format!("CredentialDefinition for cred_def_id: {:?} already exists", cred_def_id))));
        };

        let (credential_definition_value, cred_priv_key, cred_key_correctness_proof) =
            self.anoncreds_service.issuer.new_credential_definition(issuer_did, &schema, cred_def_config.support_revocation)?;

        let cred_def =
            CredentialDefinition::CredentialDefinitionV1(
                CredentialDefinitionV1 {
                    id: cred_def_id.clone(),
                    schema_id,
                    signature_type,
                    tag: tag.to_string(),
                    value: credential_definition_value
                });

        let cred_def_json = self._wallet_set_cred_def(wallet_handle, &cred_def_id, &cred_def)?;
        self._wallet_set_cred_priv_key(wallet_handle, &cred_def_id, &cred_priv_key)?;
        self._wallet_set_cred_correctness_proof(wallet_handle, &cred_def_id, &cred_key_correctness_proof)?;

        self._wallet_set_schema_id(wallet_handle, &cred_def_id, &schema.id)?; // TODO: FIXME

        trace!("create_and_store_credential_definition <<< cred_def_id: {:?}, cred_def_json: {:?}", cred_def_id, cred_def_json);
        Ok((cred_def_id, cred_def_json))
    }

    fn create_and_store_revocation_registry(&self,
                                            wallet_handle: i32,
                                            issuer_did: &str,
                                            type_: Option<&str>,
                                            tag: &str,
                                            cred_def_id: &str,
                                            config_json: &str,
                                            tails_writer_handle: i32) -> Result<(String, String, String), IndyError> {
        trace!("create_and_store_revocation_registry >>> wallet_handle: {:?}, issuer_did: {:?}, type_: {:?}, tag: {:?}, cred_def_id: {:?}, config_json: {:?}, \
               tails_handle: {:?}", wallet_handle, issuer_did, type_, tag, cred_def_id, config_json, tails_writer_handle);

        let rev_reg_config: RevocationRegistryConfig = RevocationRegistryConfig::from_json(config_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize RevocationRegistryConfig: {:?}", err)))?;

        let rev_reg_type = match type_ {
            Some(type_) =>
                RegistryType::from_json(&format!("\"{}\"", type_))
                    .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize RevocationRegistryType: {:?}", err)))?,
            None => RegistryType::CL_ACCUM,
        };

        let issuance_type = match rev_reg_config.issuance_type {
            Some(type_) =>
                IssuanceType::from_json(&format!("\"{}\"", type_))
                    .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize IssuanceType: {:?}", err)))?,
            None => IssuanceType::ISSUANCE_ON_DEMAND,
        };

        let max_cred_num = rev_reg_config.max_cred_num.unwrap_or(100000);

        let rev_reg_id = RevocationRegistryDefinition::rev_reg_id(issuer_did, cred_def_id, &rev_reg_type, tag);

        let cred_def: CredentialDefinition = self._wallet_get_cred_def(wallet_handle, &cred_def_id)?;

        let (revoc_public_keys, revoc_key_private, revoc_registry, mut revoc_tails_generator) =
            self.anoncreds_service.issuer.new_revocation_registry(&CredentialDefinitionV1::from(cred_def),
                                                                  max_cred_num,
                                                                  issuance_type.to_bool(),
                                                                  issuer_did)?;

        let (tails_location, tails_hash) =
            store_tails_from_generator(self.blob_storage_service.clone(), tails_writer_handle, &mut revoc_tails_generator)?;

        let revoc_reg_def_value = RevocationRegistryDefinitionValue {
            max_cred_num,
            issuance_type: issuance_type.clone(),
            public_keys: revoc_public_keys,
            tails_location,
            tails_hash,
        };

        let revoc_reg_def =
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(
                RevocationRegistryDefinitionV1 {
                    id: rev_reg_id.clone(),
                    type_: rev_reg_type,
                    tag: tag.to_string(),
                    cred_def_id: cred_def_id.to_string(),
                    value: revoc_reg_def_value
                });

        let revoc_reg =
            RevocationRegistry::RevocationRegistryV1(
                RevocationRegistryV1 {
                    value: revoc_registry
                }
            );

        let revoc_reg_def_json = self._wallet_set_rev_reg_def(wallet_handle, &rev_reg_id, &revoc_reg_def)?;
        let revoc_reg_json = self._wallet_set_rev_reg(wallet_handle, &rev_reg_id, &revoc_reg)?;
        self._wallet_set_rev_key_private(wallet_handle, &rev_reg_id, &revoc_key_private)?;

        let rev_reg_info = RevocationRegistryInfo {
            id: rev_reg_id.clone(),
            curr_id: 0,
            used_ids: HashSet::new(),
        };

        self._wallet_set_rev_reg_info(wallet_handle, &rev_reg_id, &rev_reg_info)?;

        trace!("create_and_store_revocation_registry <<< rev_reg_id: {:?}, revoc_reg_def_json: {:?}, revoc_reg_json: {:?}",
               rev_reg_id, revoc_reg_def_json, revoc_reg_json);

        Ok((rev_reg_id, revoc_reg_def_json, revoc_reg_json))
    }

    fn create_credential_offer(&self,
                               wallet_handle: i32,
                               cred_def_id: &str) -> Result<String, IndyError> {
        trace!("create_credential_offer >>> wallet_handle: {:?}, cred_def_id: {:?}", wallet_handle, cred_def_id);

        let key_correctness_proof: CredentialKeyCorrectnessProof = self._wallet_get_cred_correctness_proof(wallet_handle, &cred_def_id)?;

        let nonce = new_nonce()
            .map_err(|err| IndyError::AnoncredsError(AnoncredsError::from(err)))?;

        let schema_id = self._wallet_get_schema_id(wallet_handle, &cred_def_id)?; // TODO: FIXME

        let credential_offer = CredentialOffer {
            schema_id: schema_id.to_string(),
            cred_def_id: cred_def_id.to_string(),
            key_correctness_proof,
            nonce
        };

        let credential_offer_json = credential_offer.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize CredentialOffer: {:?}", err)))?;

        trace!("create_credential_offer <<< credential_offer_json: {:?}", credential_offer_json);

        Ok(credential_offer_json)
    }

    fn new_credential(&self,
                      wallet_handle: i32,
                      cred_offer_json: &str,
                      cred_req_json: &str,
                      cred_values_json: &str,
                      rev_reg_id: Option<&str>,
                      blob_storage_reader_handle: Option<i32>) -> Result<(String, Option<String>, Option<String>), IndyError> {
        trace!("new_credential >>> wallet_handle: {:?}, cred_offer_json: {:?}, cred_req_json: {:?}, cred_values_json: {:?}, rev_reg_id: {:?}, blob_storage_reader_handle: {:?}",
               wallet_handle, cred_offer_json, cred_req_json, cred_values_json, rev_reg_id, blob_storage_reader_handle);

        let cred_offer: CredentialOffer = CredentialOffer::from_json(cred_offer_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize CredentialOffer: {:?}", err)))?;

        let cred_request: CredentialRequest = CredentialRequest::from_json(cred_req_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize CredentialRequest: {:?}", err)))?;

        let cred_values: HashMap<String, AttributeValues> = serde_json::from_str(cred_values_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize CredentialValues: {:?}", err)))?;

        let cred_def: CredentialDefinitionV1 =
            CredentialDefinitionV1::from(self._wallet_get_cred_def(wallet_handle, &cred_offer.cred_def_id)?);

        let cred_priv_key: CredentialPrivateKey = self._wallet_get_cred_priv_key(wallet_handle, &cred_request.cred_def_id)?;

        let schema_id = self._wallet_get_schema_id(wallet_handle, &cred_offer.cred_def_id)?;  // TODO: FIXME

        let (rev_reg_def, mut rev_reg,
            rev_key_priv, sdk_tails_accessor, rev_reg_info) = match rev_reg_id {
            Some(ref r_reg_id) => {
                let rev_reg_def: RevocationRegistryDefinitionV1 =
                    RevocationRegistryDefinitionV1::from(
                        self._wallet_get_rev_reg_def(wallet_handle, &r_reg_id)?);

                let rev_reg: RevocationRegistryV1 =
                    RevocationRegistryV1::from(
                        self._wallet_get_rev_reg(wallet_handle, &r_reg_id)?);

                let rev_key_priv: RevocationKeyPrivate = self._wallet_get_rev_key_private(wallet_handle, &r_reg_id)?;

                let mut rev_reg_info = self._wallet_get_rev_reg_info(wallet_handle, &r_reg_id)?;

                rev_reg_info.curr_id = 1 + rev_reg_info.curr_id;

                if rev_reg_info.curr_id > rev_reg_def.value.max_cred_num {
                    return Err(IndyError::AnoncredsError(AnoncredsError::RevocationRegistryFull(format!("RevocationRegistryAccumulator is full"))));
                }

                if rev_reg_def.value.issuance_type == IssuanceType::ISSUANCE_ON_DEMAND {
                    rev_reg_info.used_ids.insert(rev_reg_info.curr_id.clone());
                }

                let blob_storage_reader_handle = blob_storage_reader_handle
                    .ok_or(IndyError::CommonError(CommonError::InvalidStructure(format!("TailsReaderHandle not found"))))?;

                let sdk_tails_accessor = SDKTailsAccessor::new(self.blob_storage_service.clone(),
                                                               blob_storage_reader_handle,
                                                               &rev_reg_def)?;

                (Some(rev_reg_def), Some(rev_reg), Some(rev_key_priv), Some(sdk_tails_accessor), Some(rev_reg_info))
            }
            None => (None, None, None, None, None)
        };

        let (credential_signature, signature_correctness_proof, rev_reg_delta) =
            self.anoncreds_service.issuer.new_credential(&cred_def,
                                                         &cred_priv_key,
                                                         &cred_offer.nonce,
                                                         &cred_request,
                                                         &cred_values,
                                                         rev_reg_info.as_ref().map(|r_reg_info| r_reg_info.curr_id),
                                                         rev_reg_def.as_ref(),
                                                         rev_reg.as_mut().map(|r_reg| &mut r_reg.value),
                                                         rev_key_priv.as_ref(),
                                                         sdk_tails_accessor.as_ref())?;

        let witness =
            if let (&Some(ref r_reg_def), &Some(ref r_reg), &Some(ref rev_tails_accessor), &Some(ref rev_reg_info)) =
            (&rev_reg_def, &rev_reg, &sdk_tails_accessor, &rev_reg_info) {
                let (issued, revoked) = match r_reg_def.value.issuance_type {
                    IssuanceType::ISSUANCE_ON_DEMAND => (rev_reg_info.used_ids.clone(), HashSet::new()),
                    IssuanceType::ISSUANCE_BY_DEFAULT => (HashSet::new(), rev_reg_info.used_ids.clone())
                };

                let rev_reg_delta = CryptoRevocationRegistryDelta::from_parts(None, &r_reg.value, &issued, &revoked);

                let witness = Some(Witness::new(rev_reg_info.curr_id, r_reg_def.value.max_cred_num, r_reg_def.value.issuance_type.to_bool(), &rev_reg_delta, rev_tails_accessor)
                    .map_err(|err| IndyError::CommonError(CommonError::from(err)))?);

                witness
            } else {
                None
            };

        let credential = Credential {
            schema_id,
            cred_def_id: cred_request.cred_def_id.clone(),
            rev_reg_id: rev_reg_id.map(String::from),
            values: cred_values,
            signature: credential_signature,
            signature_correctness_proof,
            rev_reg: rev_reg.map(|r_reg| r_reg.value),
            witness
        };

        let cred_json = credential.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize Credential: {:?}", err)))?;

        let rev_reg_delta_json = match rev_reg_delta {
            Some(r_reg_delta) => {
                Some(RevocationRegistryDelta::RevocationRegistryDeltaV1(RevocationRegistryDeltaV1 { value: r_reg_delta })
                    .to_json()
                    .map_err(|err| CommonError::InvalidState(format!("Cannot serialize RevocationRegistryDelta: {:?}", err)))?)
            }
            None => None
        };

        if let (Some(r_reg), Some(r_reg_id), Some(r_reg_info)) = (credential.rev_reg, rev_reg_id, rev_reg_info.clone()) {
            let revoc_reg = RevocationRegistry::RevocationRegistryV1(RevocationRegistryV1 { value: r_reg });

            self._wallet_update_rev_reg(wallet_handle, &r_reg_id, &revoc_reg)?;
            self._wallet_update_rev_reg_info(wallet_handle, &r_reg_id, &r_reg_info)?;
        };

        let cred_rev_id = rev_reg_info.map(|r_reg_info| r_reg_info.curr_id.to_string());
        trace!("new_credential <<< cred_json: {:?}, cred_rev_id: {:?}, rev_reg_delta_json: {:?}", cred_json, cred_rev_id, rev_reg_delta_json);

        Ok((cred_json, cred_rev_id, rev_reg_delta_json))
    }

    fn revoke_credential(&self,
                         wallet_handle: i32,
                         blob_storage_reader_handle: i32,
                         rev_reg_id: &str,
                         cred_revoc_id: &str) -> Result<String, IndyError> {
        trace!("revoke_credential >>> wallet_handle: {:?}, blob_storage_reader_handle:  {:?}, rev_reg_id: {:?}, cred_revoc_id: {:?}",
               wallet_handle, blob_storage_reader_handle, rev_reg_id, cred_revoc_id);

        let cred_revoc_id = parse_cred_rev_id(cred_revoc_id)?;

        let revocation_registry_definition: RevocationRegistryDefinitionV1 =
            RevocationRegistryDefinitionV1::from(
                self._wallet_get_rev_reg_def(wallet_handle, &rev_reg_id)?);

        let mut rev_reg: RevocationRegistryV1 =
            RevocationRegistryV1::from(
                self._wallet_get_rev_reg(wallet_handle, &rev_reg_id)?);

        let sdk_tails_accessor = SDKTailsAccessor::new(self.blob_storage_service.clone(),
                                                       blob_storage_reader_handle,
                                                       &revocation_registry_definition)?;

        if cred_revoc_id > revocation_registry_definition.value.max_cred_num + 1 {
            return Err(IndyError::AnoncredsError(AnoncredsError::InvalidUserRevocId(format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id))));
        }

        let mut rev_reg_info = self._wallet_get_rev_reg_info(wallet_handle, &rev_reg_id)?;

        match revocation_registry_definition.value.issuance_type {
            IssuanceType::ISSUANCE_ON_DEMAND => {
                if !rev_reg_info.used_ids.remove(&cred_revoc_id) {
                    return Err(IndyError::AnoncredsError(AnoncredsError::InvalidUserRevocId(format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id))));
                };
            }
            IssuanceType::ISSUANCE_BY_DEFAULT => {
                if !rev_reg_info.used_ids.insert(cred_revoc_id) {
                    return Err(IndyError::AnoncredsError(AnoncredsError::InvalidUserRevocId(format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id))));
                }
            }
        };

        let rev_reg_delta =
            self.anoncreds_service.issuer.revoke(&mut rev_reg.value, revocation_registry_definition.value.max_cred_num, cred_revoc_id, &sdk_tails_accessor)?;

        let rev_reg_delta = RevocationRegistryDelta::RevocationRegistryDeltaV1(RevocationRegistryDeltaV1 { value: rev_reg_delta });

        let rev_reg_delta_json = rev_reg_delta.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize RevocationRegistryDelta: {:?}", err)))?;

        let rev_reg = RevocationRegistry::RevocationRegistryV1(rev_reg);

        self._wallet_update_rev_reg_info(wallet_handle, &rev_reg_id, &rev_reg_info)?;
        self._wallet_update_rev_reg(wallet_handle, &rev_reg_id, &rev_reg)?;

        trace!("revoke_credential <<< rev_reg_delta_json: {:?}", rev_reg_delta_json);

        Ok(rev_reg_delta_json)
    }

    fn _recovery_credential(&self,
                            wallet_handle: i32,
                            blob_storage_reader_handle: i32,
                            rev_reg_id: &str,
                            cred_revoc_id: &str) -> Result<String, IndyError> {
        trace!("recovery_credential >>> wallet_handle: {:?}, blob_storage_reader_handle: {:?}, rev_reg_id: {:?}, cred_revoc_id: {:?}",
               wallet_handle, blob_storage_reader_handle, rev_reg_id, cred_revoc_id);

        let cred_revoc_id = parse_cred_rev_id(cred_revoc_id)?;

        let revocation_registry_definition: RevocationRegistryDefinitionV1 =
            RevocationRegistryDefinitionV1::from(
                self._wallet_get_rev_reg_def(wallet_handle, &rev_reg_id)?);

        let mut rev_reg: RevocationRegistryV1 =
            RevocationRegistryV1::from(
                self._wallet_get_rev_reg(wallet_handle, &rev_reg_id)?);

        let sdk_tails_accessor = SDKTailsAccessor::new(self.blob_storage_service.clone(),
                                                       blob_storage_reader_handle,
                                                       &revocation_registry_definition)?;

        if cred_revoc_id > revocation_registry_definition.value.max_cred_num + 1 {
            return Err(IndyError::AnoncredsError(AnoncredsError::InvalidUserRevocId(format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id))));
        }

        let mut rev_reg_info = self._wallet_get_rev_reg_info(wallet_handle, &rev_reg_id)?;

        match revocation_registry_definition.value.issuance_type {
            IssuanceType::ISSUANCE_ON_DEMAND => {
                if !rev_reg_info.used_ids.insert(cred_revoc_id) {
                    return Err(IndyError::AnoncredsError(AnoncredsError::InvalidUserRevocId(format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id))));
                }
            }
            IssuanceType::ISSUANCE_BY_DEFAULT => {
                if !rev_reg_info.used_ids.remove(&cred_revoc_id) {
                    return Err(IndyError::AnoncredsError(AnoncredsError::InvalidUserRevocId(format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id))));
                }
            }
        };

        let revocation_registry_delta =
            self.anoncreds_service.issuer.recovery(&mut rev_reg.value, revocation_registry_definition.value.max_cred_num, cred_revoc_id, &sdk_tails_accessor)?;

        let rev_reg_delta = RevocationRegistryDelta::RevocationRegistryDeltaV1(RevocationRegistryDeltaV1 { value: revocation_registry_delta });

        let rev_reg_delta_json = rev_reg_delta.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize RevocationRegistryDelta: {:?}", err)))?;

        let rev_reg = RevocationRegistry::RevocationRegistryV1(rev_reg);

        self._wallet_update_rev_reg_info(wallet_handle, &rev_reg_id, &rev_reg_info)?;
        self._wallet_update_rev_reg(wallet_handle, &rev_reg_id, &rev_reg)?;

        trace!("recovery_credential <<< rev_reg_delta_json: {:?}", rev_reg_delta_json);

        Ok(rev_reg_delta_json)
    }

    fn merge_revocation_registry_deltas(&self,
                                        rev_reg_delta_json: &str,
                                        other_rev_reg_delta_json: &str) -> Result<String, IndyError> {
        trace!("merge_revocation_registry_deltas >>> rev_reg_delta_json: {:?}, other_rev_reg_delta_json: {:?}", rev_reg_delta_json, other_rev_reg_delta_json);

        let mut rev_reg_delta: RevocationRegistryDeltaV1 =
            RevocationRegistryDeltaV1::from(
                RevocationRegistryDelta::from_json(rev_reg_delta_json)
                    .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize RevocationRegistryDelta: {:?}", err)))?);

        let other_rev_reg_delta: RevocationRegistryDeltaV1 =
            RevocationRegistryDeltaV1::from(
                RevocationRegistryDelta::from_json(other_rev_reg_delta_json)
                    .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize other RevocationRegistryDelta: {:?}", err)))?);

        rev_reg_delta.value.merge(&other_rev_reg_delta.value)
            .map_err(|err| IndyError::CommonError(CommonError::from(err)))?;

        let rev_reg_delta = RevocationRegistryDelta::RevocationRegistryDeltaV1(rev_reg_delta);

        let merged_rev_reg_delta_json = rev_reg_delta.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize RevocationRegistryDelta: {:?}", err)))?;

        trace!("merge_revocation_registry_deltas <<< merged_rev_reg_delta: {:?}", merged_rev_reg_delta_json);

        Ok(merged_rev_reg_delta_json)
    }

    fn _wallet_set_cred_def(&self, wallet_handle: i32, id: &str, cred_def: &CredentialDefinition) -> Result<String, IndyError> {
        self.wallet_service.set_object(wallet_handle, "CredentialDefinition", id, cred_def, "{}")
    }

    fn _wallet_get_cred_def(&self, wallet_handle: i32, key: &str) -> Result<CredentialDefinition, IndyError> {
        self.wallet_service.get_object::<CredentialDefinition>(wallet_handle, "CredentialDefinition", &key,
                                                               WalletRecordRetrieveOptions::RETRIEVE_ID_VALUE, &mut String::new())
    }

    fn _wallet_set_cred_priv_key(&self, wallet_handle: i32, id: &str, cred_priv_key: &CredentialPrivateKey) -> Result<String, IndyError> {
        self.wallet_service.set_object(wallet_handle, "CredentialPrivateKey", id, cred_priv_key, "{}")
    }

    fn _wallet_get_cred_priv_key(&self, wallet_handle: i32, key: &str) -> Result<CredentialPrivateKey, IndyError> {
        self.wallet_service.get_object::<CredentialPrivateKey>(wallet_handle, "CredentialPrivateKey", &key,
                                                               WalletRecordRetrieveOptions::RETRIEVE_ID_VALUE, &mut String::new())
    }

    fn _wallet_set_cred_correctness_proof(&self, wallet_handle: i32, id: &str, cred_key_correctness_proof: &CredentialKeyCorrectnessProof) -> Result<String, IndyError> {
        self.wallet_service.set_object(wallet_handle, "CredentialKeyCorrectnessProof", id, cred_key_correctness_proof, "{}")
    }

    fn _wallet_get_cred_correctness_proof(&self, wallet_handle: i32, key: &str) -> Result<CredentialKeyCorrectnessProof, IndyError> {
        self.wallet_service.get_object::<CredentialKeyCorrectnessProof>(wallet_handle, "CredentialKeyCorrectnessProof", &key,
                                                                        WalletRecordRetrieveOptions::RETRIEVE_ID_VALUE, &mut String::new())
    }

    // TODO: DELETE IT
    fn _wallet_set_schema_id(&self, wallet_handle: i32, id: &str, schema_id: &str) -> Result<(), WalletError> {
        self.wallet_service.add_record(wallet_handle, "SchemaId", id, schema_id, "{}")
    }

    // TODO: DELETE IT
    fn _wallet_get_schema_id(&self, wallet_handle: i32, key: &str) -> Result<String, IndyError> {
        let schema_id_record = self.wallet_service.get_record(wallet_handle, "SchemaId", &key, WalletRecordRetrieveOptions::RETRIEVE_ID_VALUE)?;
        Ok(schema_id_record.get_value()?.to_string())
    }

    fn _wallet_set_rev_reg_def(&self, wallet_handle: i32, id: &str, rev_reg_def: &RevocationRegistryDefinition) -> Result<String, IndyError> {
        self.wallet_service.set_object(wallet_handle, "RevocationRegistryDefinition", id, rev_reg_def, "{}")
    }

    fn _wallet_get_rev_reg_def(&self, wallet_handle: i32, key: &str) -> Result<RevocationRegistryDefinition, IndyError> {
        self.wallet_service.get_object::<RevocationRegistryDefinition>(wallet_handle, "RevocationRegistryDefinition", &key,
                                                                       WalletRecordRetrieveOptions::RETRIEVE_ID_VALUE, &mut String::new())
    }

    fn _wallet_set_rev_reg(&self, wallet_handle: i32, id: &str, rev_reg: &RevocationRegistry) -> Result<String, IndyError> {
        self.wallet_service.set_object(wallet_handle, "RevocationRegistry", id, rev_reg, "{}")
    }

    fn _wallet_update_rev_reg(&self, wallet_handle: i32, id: &str, rev_reg: &RevocationRegistry) -> Result<String, IndyError> {
        self.wallet_service.update_object(wallet_handle, "RevocationRegistry", id, rev_reg)
    }

    fn _wallet_get_rev_reg(&self, wallet_handle: i32, key: &str) -> Result<RevocationRegistry, IndyError> {
        self.wallet_service.get_object::<RevocationRegistry>(wallet_handle, "RevocationRegistry", &key,
                                                             WalletRecordRetrieveOptions::RETRIEVE_ID_VALUE, &mut String::new())
    }

    fn _wallet_set_rev_key_private(&self, wallet_handle: i32, id: &str, rev_key_priv: &RevocationKeyPrivate) -> Result<String, IndyError> {
        self.wallet_service.set_object(wallet_handle, "RevocationKeyPrivate", id, rev_key_priv, "{}")
    }

    fn _wallet_get_rev_key_private(&self, wallet_handle: i32, key: &str) -> Result<RevocationKeyPrivate, IndyError> {
        self.wallet_service.get_object::<RevocationKeyPrivate>(wallet_handle, "RevocationKeyPrivate", &key,
                                                               WalletRecordRetrieveOptions::RETRIEVE_ID_VALUE, &mut String::new())
    }

    fn _wallet_set_rev_reg_info(&self, wallet_handle: i32, id: &str, rev_reg_info: &RevocationRegistryInfo) -> Result<String, IndyError> {
        self.wallet_service.set_object(wallet_handle, "RevocationRegistryInfo", id, rev_reg_info, "{}")
    }

    fn _wallet_update_rev_reg_info(&self, wallet_handle: i32, id: &str, rev_reg_info: &RevocationRegistryInfo) -> Result<String, IndyError> {
        self.wallet_service.update_object(wallet_handle, "RevocationRegistryInfo", id, rev_reg_info)
    }

    fn _wallet_get_rev_reg_info(&self, wallet_handle: i32, key: &str) -> Result<RevocationRegistryInfo, IndyError> {
        self.wallet_service.get_object::<RevocationRegistryInfo>(wallet_handle, "RevocationRegistryInfo", &key,
                                                                 WalletRecordRetrieveOptions::RETRIEVE_ID_VALUE, &mut String::new())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct RevocationRegistryInfo {
    id: String,
    curr_id: u32,
    used_ids: HashSet<u32>
}

impl JsonEncodable for RevocationRegistryInfo {}

impl<'a> JsonDecodable<'a> for RevocationRegistryInfo {}