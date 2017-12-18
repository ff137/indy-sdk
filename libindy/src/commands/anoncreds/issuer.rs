extern crate serde_json;
extern crate indy_crypto;

use errors::indy::IndyError;
use errors::common::CommonError;

use services::anoncreds::AnoncredsService;
use services::pool::PoolService;
use services::wallet::WalletService;
use services::anoncreds::types::*;
use services::anoncreds::helpers::get_composite_id;
use std::rc::Rc;
use std::collections::HashMap;
use self::indy_crypto::cl::*;
use self::indy_crypto::utils::json::{JsonDecodable, JsonEncodable};

pub enum IssuerCommand {
    CreateAndStoreClaimDefinition(
        i32, // wallet handle
        String, // issuer did
        String, // schema json
        Option<String>, // signature type
        bool,
        Box<Fn(Result<String, IndyError>) + Send>),
    CreateAndStoreRevocationRegistry(
        i32, // wallet handle
        String, // issuer did
        i32, // schema seq no
        u32, // max claim num
        Box<Fn(Result<String, IndyError>) + Send>),
    CreateClaim(
        i32, // wallet handle
        String, // claim req json
        String, // claim json
        Option<u32>, // user revoc index
        Box<Fn(Result<(String, String), IndyError>) + Send>),
    RevokeClaim(
        i32, // wallet handle
        String, // issuer did
        i32, // schema seq no
        u32, // user revoc index
        Box<Fn(Result<String, IndyError>) + Send>),
}

pub struct IssuerCommandExecutor {
    pub anoncreds_service: Rc<AnoncredsService>,
    pub pool_service: Rc<PoolService>,
    pub wallet_service: Rc<WalletService>
}

impl IssuerCommandExecutor {
    pub fn new(anoncreds_service: Rc<AnoncredsService>,
               pool_service: Rc<PoolService>,
               wallet_service: Rc<WalletService>) -> IssuerCommandExecutor {
        IssuerCommandExecutor {
            anoncreds_service,
            pool_service,
            wallet_service,
        }
    }

    pub fn execute(&self, command: IssuerCommand) {
        match command {
            IssuerCommand::CreateAndStoreClaimDefinition(wallet_handle, issuer_did, schema_json, signature_type, create_non_revoc, cb) => {
                info!(target: "issuer_command_executor", "CreateAndStoreClaim command received");
                self.create_and_store_claim_definition(wallet_handle, &issuer_did, &schema_json,
                                                       signature_type.as_ref().map(String::as_str), create_non_revoc, cb);
            }
            IssuerCommand::CreateAndStoreRevocationRegistry(wallet_handle, issuer_did, schema_seq_no, max_claim_num, cb) => {
                info!(target: "issuer_command_executor", "CreateAndStoreRevocationRegistryRegistry command received");
                self.create_and_store_revocation_registry(wallet_handle, &issuer_did, schema_seq_no, max_claim_num, cb);
            }
            IssuerCommand::CreateClaim(wallet_handle, claim_req_json, claim_json, user_revoc_index, cb) => {
                info!(target: "issuer_command_executor", "CreateClaim command received");
                self.new_claim(wallet_handle, &claim_req_json, &claim_json, user_revoc_index, cb);
            }
            IssuerCommand::RevokeClaim(wallet_handle, issuer_did, schema_seq_no,
                                       user_revoc_index, cb) => {
                info!(target: "issuer_command_executor", "RevokeClaim command received");
                self.revoke_claim(wallet_handle, &issuer_did, schema_seq_no, user_revoc_index, cb);
            }
        };
    }

    fn create_and_store_claim_definition(&self,
                                         wallet_handle: i32,
                                         issuer_did: &str,
                                         schema_json: &str,
                                         signature_type: Option<&str>,
                                         create_non_revoc: bool,
                                         cb: Box<Fn(Result<String, IndyError>) + Send>) {
        let result = self._create_and_store_claim_definition(wallet_handle, issuer_did, schema_json,
                                                             signature_type, create_non_revoc);
        cb(result)
    }

    fn _create_and_store_claim_definition(&self,
                                          wallet_handle: i32,
                                          issuer_did: &str,
                                          schema_json: &str,
                                          signature_type: Option<&str>,
                                          create_non_revoc: bool) -> Result<String, IndyError> {
        info!("create_and_store_claim_definition >>> wallet_handle: {:?}, issuer_did: {:?}, schema_json: {:?}, \
                       signature_type: {:?}, create_non_revoc: {:?}", wallet_handle, issuer_did, schema_json, signature_type, create_non_revoc);

        let schema: Schema = Schema::from_json(schema_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Invalid schema json: {}", err.to_string())))?;

        let (claim_definition, private_key) =
            self.anoncreds_service.issuer.new_claim_definition(issuer_did, &schema, signature_type, create_non_revoc)?;

        let claim_definition_json = claim_definition.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize claim definition: {:?}", err)))?;

        let private_key_json = private_key.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize claim definition private key: {:?}", err)))?;

        let id = get_composite_id(issuer_did, schema.seq_no);
        self.wallet_service.set(wallet_handle, &format!("claim_definition::{}", id), &claim_definition_json)?;
        self.wallet_service.set(wallet_handle, &format!("claim_definition_private_key::{}", id), &private_key_json)?;

        info!("create_and_store_claim_definition <<< claim_definition_json: {:?}", claim_definition_json);

        Ok(claim_definition_json)
    }

    fn create_and_store_revocation_registry(&self,
                                            wallet_handle: i32,
                                            issuer_did: &str,
                                            schema_seq_no: i32,
                                            max_claim_num: u32,
                                            cb: Box<Fn(Result<String, IndyError>) + Send>) {
        let result = self._create_and_store_revocation_registry(wallet_handle, issuer_did, schema_seq_no, max_claim_num);
        cb(result)
    }

    fn _create_and_store_revocation_registry(&self,
                                             wallet_handle: i32,
                                             issuer_did: &str,
                                             schema_seq_no: i32,
                                             max_claim_num: u32) -> Result<String, IndyError> {
        info!("create_and_store_revocation_registry >>> wallet_handle: {:?}, issuer_did: {:?}, schema_seq_no: {:?}, max_claim_num: {:?}",
              wallet_handle, issuer_did, schema_seq_no, max_claim_num);

        let claim_def_json = self.wallet_service.get(wallet_handle, &format!("claim_definition::{}", &get_composite_id(issuer_did, schema_seq_no)))?;
        let claim_def: ClaimDefinition = ClaimDefinition::from_json(&claim_def_json)
            .map_err(|err| CommonError::InvalidState(format!("Cannon deserialize claim definition: {:?}", err)))?;

        let (revocation_registry, revocation_registry_private) =
            self.anoncreds_service.issuer.new_revocation_registry(&claim_def.data, max_claim_num, issuer_did, schema_seq_no)?;

        let revocation_registry_json = revocation_registry.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannon serialize revocation registry: {:?}", err)))?;

        let revocation_registry_private_json = revocation_registry_private.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannon serialize revocation registry private: {:?}", err)))?;

        let id = get_composite_id(issuer_did, schema_seq_no);
        self.wallet_service.set(wallet_handle, &format!("revocation_registry::{}", id), &revocation_registry_json)?;
        self.wallet_service.set(wallet_handle, &format!("revocation_registry_private::{}", id), &revocation_registry_private_json)?;

        // TODO: decide about tails storing
        info!("create_and_store_revocation_registry <<< revocation_registry_json: {:?}", revocation_registry_json);

        Ok(revocation_registry_json)
    }

    fn new_claim(&self,
                 wallet_handle: i32,
                 claim_req_json: &str,
                 claim_json: &str,
                 user_revoc_index: Option<u32>,
                 cb: Box<Fn(Result<(String, String), IndyError>) + Send>) {
        let result = self._new_claim(wallet_handle, claim_req_json, claim_json, user_revoc_index);
        cb(result)
    }

    fn _new_claim(&self,
                  wallet_handle: i32,
                  claim_req_json: &str,
                  claim_json: &str,
                  rev_idx: Option<u32>) -> Result<(String, String), IndyError> {
        info!("new_claim >>> wallet_handle: {:?}, claim_req_json: {:?}, claim_json: {:?}, rev_idx: {:?}",
              wallet_handle, claim_req_json, claim_json, rev_idx);

        let claim_request: ClaimRequest = ClaimRequest::from_json(claim_req_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize claim request: {:?}", err)))?;

        let id = get_composite_id(&claim_request.issuer_did, claim_request.schema_seq_no);

        let claim_def_json = self.wallet_service.get(wallet_handle, &format!("claim_definition::{}", id))?;
        let claim_def: ClaimDefinition = ClaimDefinition::from_json(&claim_def_json)
            .map_err(|err| CommonError::InvalidState(format!("Cannot deserialize claim definition: {:?}", err)))?;

        let private_key_json = self.wallet_service.get(wallet_handle, &format!("claim_definition_private_key::{}", id))?;
        let private_key = IssuerPrivateKey::from_json(&private_key_json)
            .map_err(|err| CommonError::InvalidState(format!("Cannot deserialize claim definition private key: {:?}", err)))?;

        let mut rev_reg_pub = match self.wallet_service.get(wallet_handle, &format!("revocation_registry::{}", id)) {
            Ok(rev_reg_pub_json) =>
                Some(RevocationRegistry::from_json(&rev_reg_pub_json)
                    .map_err(|err| CommonError::InvalidState(format!("Cannon deserialize revocation registry: {:?}", err)))?),
            Err(_) => None
        };

        let rev_reg_priv = match self.wallet_service.get(wallet_handle, &format!("revocation_registry_private::{}", id)) {
            Ok(rev_reg_priv_json) =>
                Some(RevocationRegistryPrivate::from_json(&rev_reg_priv_json)
                    .map_err(|err| CommonError::InvalidState(format!("Cannon deserialize revocation registry private: {:?}", err)))?),
            Err(_) => None
        };

        let claim_values: HashMap<String, Vec<String>> = serde_json::from_str(claim_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannon deserialize claim values: {:?}", err)))?;

        let claim_signature = self.anoncreds_service.issuer.new_claim(&claim_def.data,
                                                                      &private_key,
                                                                      rev_reg_pub.as_mut(),
                                                                      rev_reg_priv.as_ref(),
                                                                      &claim_request,
                                                                      &claim_values,
                                                                      rev_idx)?;

        let revocation_registry_json = match rev_reg_pub {
            Some(rev_reg) => {
                let rev_reg_json = rev_reg.to_json()
                    .map_err(|err| CommonError::InvalidState(format!("Cannon serialize revocation registry: {:?}", err)))?;

                let id = get_composite_id(&claim_request.issuer_did, claim_request.schema_seq_no);
                self.wallet_service.set(wallet_handle, &format!("revocation_registry::{}", &id), &rev_reg_json)?;

                rev_reg_json
            }
            None => String::new()
        };

        let claim = Claim {
            values: claim_values,
            signature: claim_signature,
            schema_seq_no: claim_request.schema_seq_no,
            issuer_did: claim_request.issuer_did,
            rev_reg_seq_no: None // TODO: How Issuer gets rev_reg_seq_no
        };

        let claim_json = claim.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize claim: {:?}", err)))?;

        info!("new_claim <<< revocation_registry_json: {:?}, claim_json: {:?}", revocation_registry_json, claim_json);

        Ok((revocation_registry_json, claim_json))
    }

    fn revoke_claim(&self,
                    wallet_handle: i32,
                    issuer_did: &str,
                    schema_seq_no: i32,
                    user_revoc_index: u32,
                    cb: Box<Fn(Result<String, IndyError>) + Send>) {
        let result = self._revoke_claim(wallet_handle, issuer_did, schema_seq_no, user_revoc_index);
        cb(result)
    }

    fn _revoke_claim(&self,
                     wallet_handle: i32,
                     issuer_did: &str,
                     schema_seq_no: i32,
                     user_revoc_index: u32) -> Result<String, IndyError> {
        info!("revoke_claim >>> wallet_handle: {:?}, issuer_did: {:?}, schema_seq_no: {:?}, user_revoc_index: {:?}",
              wallet_handle, issuer_did, schema_seq_no, user_revoc_index);

        let id = get_composite_id(&issuer_did, schema_seq_no);

        let revocation_registry_json = self.wallet_service.get(wallet_handle, &format!("revocation_registry::{}", id))?;
        let mut revocation_registry = RevocationRegistry::from_json(&revocation_registry_json)
            .map_err(|err| CommonError::InvalidState(format!("Cannon deserialize revocation registry: {:?}", err)))?;

        let revocation_registry_private_json = self.wallet_service.get(wallet_handle, &format!("revocation_registry_private::{}", id))?;
        let revocation_registry_private = RevocationRegistryPrivate::from_json(&revocation_registry_private_json)
            .map_err(|err| CommonError::InvalidState(format!("Cannon deserialize  revocation registry private: {:?}", err)))?;

        self.anoncreds_service.issuer.revoke(&mut revocation_registry, user_revoc_index)?;

        let revocation_registry_updated_json = revocation_registry.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannon serialize revocation registry: {:?}", err)))?;

        self.wallet_service.set(wallet_handle, &format!("revocation_registry::{}", id), &revocation_registry_updated_json)?;

        info!("revoke_claim <<< revocation_registry_updated_json: {:?}", revocation_registry_updated_json);

        Ok(revocation_registry_updated_json)
    }
}
