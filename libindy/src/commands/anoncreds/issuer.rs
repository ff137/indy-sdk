extern crate serde_json;
extern crate indy_crypto;

use errors::indy::IndyError;
use errors::anoncreds::AnoncredsError;
use errors::common::CommonError;

use services::anoncreds::AnoncredsService;
use services::blob_storage::BlobStorageService;
use services::pool::PoolService;
use services::wallet::WalletService;
use services::crypto::CryptoService;
use services::anoncreds::types::*;
use services::anoncreds::helpers::get_composite_id;
use std::rc::Rc;
use std::collections::HashMap;
use self::indy_crypto::cl::*;
use self::indy_crypto::utils::json::{JsonDecodable, JsonEncodable};
use super::tails::{SDKTailsAccessor, store_tails_from_generator};

pub enum IssuerCommand {
    CreateAndStoreCredentialDefinition(
        i32, // wallet handle
        String, // issuer did
        String, // schema json
        Option<String>, // signature type
        bool, // support revocation
        Box<Fn(Result<String, IndyError>) + Send>),
    CreateAndStoreRevocationRegistry(
        i32, // wallet handle
        String, // tails writer type
        String, // tails writer config
        String, // schema json
        String, // issuer did
        u32, // max credential num
        bool, // issuance by default
        Box<Fn(Result<(String, String), IndyError>) + Send>),
    CreateCredentialOffer(
        i32, // wallet handle
        String, // issuer did
        String, // schema json
        String, // prover did
        Box<Fn(Result<String, IndyError>) + Send>),
    CreateClaim(
        i32, // wallet handle
        String, // credential req json
        String, // credential json
        Option<i32>, // tails reader handle
        Option<u32>, // user revoc index
        Box<Fn(Result<(Option<String>, String), IndyError>) + Send>),
    RevokeClaim(
        i32, // wallet handle
        i32, // tails reader handle
        String, // issuer did
        String, // schema json
        u32, // user revoc index
        Box<Fn(Result<String, IndyError>) + Send>),
    RecoverClaim(
        i32, // wallet handle
        i32, // tails reader handle
        String, // issuer did
        String, // schema json
        u32, // user revoc index
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
            IssuerCommand::CreateAndStoreCredentialDefinition(wallet_handle, issuer_did, schema_json, signature_type, create_non_revoc, cb) => {
                trace!(target: "issuer_command_executor", "CreateAndStoreClaimDef command received");
                cb(self.create_and_store_credential_definition(wallet_handle, &issuer_did, &schema_json,
                                                               signature_type.as_ref().map(String::as_str), create_non_revoc));
            }
            IssuerCommand::CreateAndStoreRevocationRegistry(wallet_handle, tails_writer_type, tails_writer_config, issuer_did, schema_json, max_cred_num, issuance_by_default, cb) => {
                trace!(target: "issuer_command_executor", "CreateAndStoreRevocationRegistryRegistry command received");
                cb(self.create_and_store_revocation_registry(wallet_handle, &tails_writer_type, &tails_writer_config, &issuer_did, &schema_json, max_cred_num, issuance_by_default));
            }
            IssuerCommand::CreateClaim(wallet_handle, credential_req_json, credential_json, tails_reader_handle, rev_idx, cb) => {
                info!(target: "issuer_command_executor", "CreateClaim command received");
                cb(self.new_credential(wallet_handle, &credential_req_json, &credential_json, tails_reader_handle, rev_idx));
            }
            IssuerCommand::CreateCredentialOffer(wallet_handle, schema_json, issuer_did, prover_did, cb) => {
                trace!(target: "issuer_command_executor", "CreateCredentialOffer command received");
                cb(self.create_credential_offer(wallet_handle, &schema_json, &issuer_did, &prover_did));
            }
            IssuerCommand::RevokeClaim(wallet_handle, tails_reader_handle, issuer_did, schema_json, user_revoc_index, cb) => {
                trace!(target: "issuer_command_executor", "RevokeClaim command received");
                cb(self.revoke_credential(wallet_handle, tails_reader_handle, &issuer_did, &schema_json, user_revoc_index));
            }
            IssuerCommand::RecoverClaim(wallet_handle, tails_reader_handle, issuer_did, schema_json, user_revoc_index, cb) => {
                trace!(target: "issuer_command_executor", "RecoverClaim command received");
                cb(self.recovery_credential(wallet_handle, tails_reader_handle, &issuer_did, &schema_json, user_revoc_index));
            }
        };
    }

    fn create_and_store_credential_definition(&self,
                                              wallet_handle: i32,
                                              issuer_did: &str,
                                              schema_json: &str,
                                              signature_type: Option<&str>,
                                              support_revocation: bool) -> Result<String, IndyError> {
        trace!("create_and_store_credential_definition >>> wallet_handle: {:?}, issuer_did: {:?}, schema_json: {:?}, \
              signature_type: {:?}, support_revocation: {:?}", wallet_handle, issuer_did, schema_json, signature_type, support_revocation);

        self.crypto_service.validate_did(issuer_did)?;

        let schema: Schema = Schema::from_json(schema_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize Schema: {:?}", err)))?;

        let id = get_composite_id(issuer_did, &schema.schema_key());

        if self.wallet_service.get(wallet_handle, &format!("credential_definition::{}", id)).is_ok() {
            return Err(IndyError::AnoncredsError(AnoncredsError::ClaimDefAlreadyExists(format!("CredentialDefinition for key: {:?} already exists", id))));
        };

        let (credential_definition, credential_priv_key, credential_key_correctness_proof) =
            self.anoncreds_service.issuer.new_credential_definition(issuer_did, &schema, signature_type, support_revocation)?;

        let credential_definition_json =
            self.wallet_service.set_object(wallet_handle, &format!("credential_definition::{}", id), &credential_definition, "CredentialDefinition")?;
        self.wallet_service.set_object(wallet_handle, &format!("credential_private_key::{}", id), &credential_priv_key, "CredentialPrivateKey")?;
        self.wallet_service.set_object(wallet_handle, &format!("credential_key_correctness_proof::{}", id), &credential_key_correctness_proof, "credential_key_correctness_proof_json")?;

        trace!("create_and_store_credential_definition <<< credential_definition_json: {:?}", credential_definition_json);

        Ok(credential_definition_json)
    }

    fn create_and_store_revocation_registry(&self,
                                            wallet_handle: i32,
                                            tails_writer_type: &str,
                                            tails_writer_config: &str,
                                            issuer_did: &str,
                                            schema_json: &str,
                                            max_cred_num: u32,
                                            issuance_by_default: bool) -> Result<(String, String), IndyError> {
        trace!("create_and_store_revocation_registry >>> wallet_handle: {:?}, tails_writer_type: {:?}, tails_writer_config: {:?}, issuer_did: {:?}, schema_json: {:?}, \
               max_cred_num: {:?}", wallet_handle, tails_writer_type, tails_writer_config, issuer_did, schema_json, max_cred_num);

        self.crypto_service.validate_did(issuer_did)?;

        let schema: Schema = Schema::from_json(schema_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize Schema: {:?}", err)))?;

        let id = get_composite_id(issuer_did, &schema.schema_key());

        let credential_def: CredentialDefinition =
            self.wallet_service.get_object(wallet_handle, &format!("credential_definition::{}", &id), "CredentialDefinition", &mut String::new())?;

        let (revocation_public_keys, revocation_key_private, revocation_registry, mut revocation_tails_generator) =
            self.anoncreds_service.issuer.new_revocation_registry(&credential_def,
                                                                  max_cred_num,
                                                                  issuance_by_default,
                                                                  issuer_did,
                                                                  schema.seq_no)?;

        let (tails_location, tails_hash) =
            store_tails_from_generator(self.blob_storage_service.clone(), tails_writer_type, tails_writer_config, &mut revocation_tails_generator)?;

        let revocation_registry_definition = RevocationRegistryDefinition {
            max_cred_num,
            issuance_type: if issuance_by_default { IssuanceTypes::ISSUANCE_BY_DEFAULT } else { IssuanceTypes::ISSUANCE_ON_DEMAND },
            public_keys: revocation_public_keys,
            tails_location,
            tails_hash,
        };

        // TODO: store revocation registry using unique identifier(https://jira.hyperledger.org/browse/IS-514).
        let revocation_registry_definition_json =
            self.wallet_service.set_object(wallet_handle, &format!("revocation_registry_definition::{}", id), &revocation_registry_definition, "RevocationRegistryDefinition")?;
        let revocation_registry_json =
            self.wallet_service.set_object(wallet_handle, &format!("revocation_registry::{}", id), &revocation_registry, "RevocationRegistry")?;
        let revocation_tails_generator_json =
            self.wallet_service.set_object(wallet_handle, &format!("revocation_tails_generator::{}", id), &revocation_tails_generator, "RevocationTailsGenerator")?;
        self.wallet_service.set_object(wallet_handle, &format!("revocation_key_private::{}", id), &revocation_key_private, "RevocationKeyPrivate")?;

        // TODO: decide about tails storing
        trace!("create_and_store_revocation_registry <<< revocation_registry_definition_json: {:?}, revocation_registry_json: {:?}",
               revocation_registry_definition_json, revocation_registry_json);

        Ok((revocation_registry_definition_json, revocation_registry_json))
    }

    fn create_credential_offer(&self,
                               wallet_handle: i32,
                               schema_json: &str,
                               issuer_did: &str,
                               prover_did: &str) -> Result<String, IndyError> {
        trace!("create_credential_offer >>> wallet_handle: {:?}, issuer_did: {:?}, schema_json: {:?}, prover_did: {:?}",
               wallet_handle, issuer_did, schema_json, prover_did);

        self.crypto_service.validate_did(issuer_did)?;
        self.crypto_service.validate_did(prover_did)?;

        let schema: Schema = Schema::from_json(schema_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Invalid schema json: {}", err.to_string())))?;

        let id = get_composite_id(issuer_did, &schema.schema_key());

        let key_correctness_proof: CredentialKeyCorrectnessProof =
            self.wallet_service.get_object(wallet_handle, &format!("credential_key_correctness_proof::{}", id), "CredentialKeyCorrectnessProof", &mut String::new())?;

        let nonce = new_nonce()
            .map_err(|err| IndyError::AnoncredsError(AnoncredsError::from(err)))?;

        self.wallet_service.set_object(wallet_handle, &format!("master_secret_blinding_nonce::{}::{}", id, prover_did), &nonce, "Nonce")?;

        let credential_offer = CredentialOffer {
            issuer_did: issuer_did.to_string(),
            schema_key: schema.schema_key(),
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
                      credential_req_json: &str,
                      credential_json: &str,
                      tails_reader_handle: Option<i32>,
                      rev_idx: Option<u32>) -> Result<(Option<String>, String), IndyError> {
        trace!("new_credential >>> wallet_handle: {:?}, tails_reader_handle: {:?}, credential_req_json: {:?}, credential_json: {:?}, rev_idx: {:?}",
               wallet_handle, tails_reader_handle, credential_req_json, credential_json, rev_idx);

        let credential_request: CredentialRequest = CredentialRequest::from_json(credential_req_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize CredentialRequest: {:?}", err)))?;

        let id = get_composite_id(&credential_request.issuer_did, &credential_request.schema_key);

        let credential_def: CredentialDefinition =
            self.wallet_service.get_object(wallet_handle, &format!("credential_definition::{}", id), "CredentialDefinition", &mut String::new())?;

        let credential_priv_key: CredentialPrivateKey =
            self.wallet_service.get_object(wallet_handle, &format!("credential_private_key::{}", id), "CredentialPrivateKey", &mut String::new())?;

        let master_secret_blinding_nonce: Nonce =
            self.wallet_service.get_object(wallet_handle, &format!("master_secret_blinding_nonce::{}::{}", id, credential_request.prover_did), "Nonce", &mut String::new())?;

        let credential_values: HashMap<String, Vec<String>> = serde_json::from_str(credential_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize CredentialValues: {:?}", err)))?;

        let rev_reg_def: Option<RevocationRegistryDefinition> =
            self.wallet_service.get_opt_object(wallet_handle, &format!("revocation_registry_definition::{}", id), "RevocationRegistryDefinition", &mut String::new())?;

        let mut rev_reg: Option<RevocationRegistry> =
            self.wallet_service.get_opt_object(wallet_handle, &format!("revocation_registry::{}", id), "RevocationRegistry", &mut String::new())?;

        let rev_key_priv: Option<RevocationKeyPrivate> =
            self.wallet_service.get_opt_object(wallet_handle, &format!("revocation_key_private::{}", id), "RevocationKeyPrivate", &mut String::new())?;

        let sdk_tails_accessor = match tails_reader_handle {
            Some(handle) => Some(SDKTailsAccessor::new(self.blob_storage_service.clone(), handle)),
            None => None
        };

        let (credential_signature, signature_correctness_proof, rev_reg_delta) =
            self.anoncreds_service.issuer.new_credential(&credential_def,
                                                         &credential_priv_key,
                                                         &master_secret_blinding_nonce,
                                                         &credential_request,
                                                         &credential_values,
                                                         rev_idx,
                                                         rev_reg_def.as_ref(),
                                                         rev_reg.as_mut(),
                                                         rev_key_priv.as_ref(),
                                                         sdk_tails_accessor.as_ref())?;

        if let Some(r_reg) = rev_reg {
            self.wallet_service.set_object(wallet_handle, &format!("revocation_registry::{}", id), &r_reg, "RevocationRegistry")?;
        }

        let credential = Credential {
            values: credential_values,
            signature: credential_signature,
            signature_correctness_proof,
            schema_key: credential_request.schema_key,
            issuer_did: credential_request.issuer_did,
            rev_reg_seq_no: None // TODO: How Issuer gets rev_reg_seq_no
        };

        let credential_json = credential.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize Credential: {:?}", err)))?;

        let rev_reg_delta_json = match rev_reg_delta {
            Some(r_reg_delta) => {
                Some(r_reg_delta.to_json()
                    .map_err(|err| CommonError::InvalidState(format!("Cannot serialize RevocationRegistryDelta: {:?}", err)))?)
            }
            None => None
        };

        trace!("new_credential <<< rev_reg_delta_json: {:?}, credential_json: {:?}", rev_reg_delta_json, credential_json);

        Ok((rev_reg_delta_json, credential_json))
    }

    fn revoke_credential(&self,
                         wallet_handle: i32,
                         tails_reader_handle: i32,
                         issuer_did: &str,
                         schema_json: &str,
                         rev_idx: u32) -> Result<String, IndyError> {
        trace!("revoke_credential >>> wallet_handle: {:?}, issuer_did: {:?}, schema_json: {:?}, rev_idx: {:?}",
               wallet_handle, issuer_did, schema_json, rev_idx);

        let schema: Schema = Schema::from_json(schema_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize Schema: {:?}", err)))?;

        let id = get_composite_id(issuer_did, &schema.schema_key());

        let revocation_registry_definition: RevocationRegistryDefinition =
            self.wallet_service.get_object(wallet_handle, &format!("revocation_registry_definition::{}", id), "RevocationRegistryDefinition", &mut String::new())?;

        let mut revocation_registry: RevocationRegistry =
            self.wallet_service.get_object(wallet_handle, &format!("revocation_registry::{}", id), "RevocationRegistry", &mut String::new())?;

        let sdk_tails_accessor = SDKTailsAccessor::new(self.blob_storage_service.clone(), tails_reader_handle);

        let revocation_registry_delta =
            self.anoncreds_service.issuer.revoke(&mut revocation_registry, revocation_registry_definition.max_cred_num, rev_idx, &sdk_tails_accessor)?;

        self.wallet_service.set_object(wallet_handle, &format!("revocation_registry::{}", id), &revocation_registry, "RevocationRegistry")?;

        let revocation_registry_delta_json = revocation_registry_delta.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize RevocationRegistryDelta: {:?}", err)))?;

        trace!("revoke_credential <<< revocation_registry_delta_json: {:?}", revocation_registry_delta_json);

        Ok(revocation_registry_delta_json)
    }

    fn recovery_credential(&self,
                           wallet_handle: i32,
                           tails_reader_handle: i32,
                           issuer_did: &str,
                           schema_json: &str,
                           rev_idx: u32) -> Result<String, IndyError> {
        trace!("recovery_credential >>> wallet_handle: {:?}, issuer_did: {:?}, schema_json: {:?}, rev_idx: {:?}",
               wallet_handle, issuer_did, schema_json, rev_idx);

        let schema: Schema = Schema::from_json(schema_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize Schema: {:?}", err)))?;

        let id = get_composite_id(issuer_did, &schema.schema_key());

        let revocation_registry_definition: RevocationRegistryDefinition =
            self.wallet_service.get_object(wallet_handle, &format!("revocation_registry_definition::{}", id), "RevocationRegistryDefinition", &mut String::new())?;

        let mut revocation_registry: RevocationRegistry =
            self.wallet_service.get_object(wallet_handle, &format!("revocation_registry::{}", id), "RevocationRegistry", &mut String::new())?;

        let sdk_tails_accessor = SDKTailsAccessor::new(self.blob_storage_service.clone(), tails_reader_handle);

        let revocation_registry_delta =
            self.anoncreds_service.issuer.recovery(&mut revocation_registry, revocation_registry_definition.max_cred_num, rev_idx, &sdk_tails_accessor)?;

        self.wallet_service.set_object(wallet_handle, &format!("revocation_registry::{}", id), &revocation_registry, "RevocationRegistry")?;

        let revocation_registry_delta_json = revocation_registry_delta.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize RevocationRegistryDelta: {:?}", err)))?;

        trace!("recovery_credential <<< revocation_registry_delta_json: {:?}", revocation_registry_delta_json);

        Ok(revocation_registry_delta_json)
    }
}
