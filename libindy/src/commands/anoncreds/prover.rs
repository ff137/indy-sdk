extern crate serde_json;
extern crate indy_crypto;

use errors::common::CommonError;
use errors::indy::IndyError;
use errors::anoncreds::AnoncredsError;
use services::anoncreds::AnoncredsService;
use services::wallet::WalletService;
use services::crypto::CryptoService;
use std::rc::Rc;
use services::anoncreds::helpers::get_composite_id;
use services::anoncreds::types::*;
use services::blob_storage::BlobStorageService;
use std::collections::{HashMap, HashSet};
use self::indy_crypto::cl::*;
use self::indy_crypto::utils::json::{JsonDecodable, JsonEncodable};
use super::tails::SDKTailsAccessor;

pub enum ProverCommand {
    StoreCredentialOffer(
        i32, // wallet handle
        String, // credential offer json
        Box<Fn(Result<(), IndyError>) + Send>),
    GetCredentialOffers(
        i32, // wallet handle
        String, // filter json
        Box<Fn(Result<String, IndyError>) + Send>),
    CreateMasterSecret(
        i32, // wallet handle
        String, // master secret name
        Box<Fn(Result<(), IndyError>) + Send>),
    CreateAndStoreCredentialRequest(
        i32, // wallet handle
        String, // prover did
        String, // credential offer json
        String, // credential def json
        String, // master secret name
        Box<Fn(Result<String, IndyError>) + Send>),
    StoreCredential(
        i32, // wallet handle
        String, // id
        String, // credentials json
        Option<String>, // revocation registry definition json
        Option<String>, // revocation registry entry json
        Box<Fn(Result<(), IndyError>) + Send>),
    GetCredentials(
        i32, // wallet handle
        String, // filter json
        Box<Fn(Result<String, IndyError>) + Send>),
    GetCredentialsForProofReq(
        i32, // wallet handle
        String, // proof request json
        Box<Fn(Result<String, IndyError>) + Send>),
    CreateProof(
        i32, // wallet handle
        String, // proof request json
        String, // requested credentials json
        String, // schemas json
        String, // master secret name
        String, // credential defs json
        String, // rev reg entries json
        Box<Fn(Result<String, IndyError>) + Send>),
    CreateWitness(
        i32, // wallet handle
        i32, // tails reader _handle
        String, // revocation registry definition json
        String, // revocation registry delta json
        u32, //rev_idx
        Box<Fn(Result<String, IndyError>) + Send>),
    UpdateWitness(
        i32, // wallet handle
        i32, // tails reader _handle
        String, // witness json
        String, // revocation registry definition json
        String, // revocation registry delta json
        u32, //rev_idx
        Box<Fn(Result<String, IndyError>) + Send>),
    StoreWitness(
        i32, // wallet handle
        String, // id
        String, // witness json
        Box<Fn(Result<(), IndyError>) + Send>),
    GetWitness(
        i32, // wallet handle
        String, // id
        Box<Fn(Result<String, IndyError>) + Send>)
}

pub struct ProverCommandExecutor {
    anoncreds_service: Rc<AnoncredsService>,
    wallet_service: Rc<WalletService>,
    crypto_service: Rc<CryptoService>,
    blob_storage_service: Rc<BlobStorageService>,
}

impl ProverCommandExecutor {
    pub fn new(anoncreds_service: Rc<AnoncredsService>,
               wallet_service: Rc<WalletService>,
               crypto_service: Rc<CryptoService>,
               blob_storage_service: Rc<BlobStorageService>) -> ProverCommandExecutor {
        ProverCommandExecutor {
            anoncreds_service,
            wallet_service,
            crypto_service,
            blob_storage_service
        }
    }

    pub fn execute(&self, command: ProverCommand) {
        match command {
            ProverCommand::StoreCredentialOffer(wallet_handle, credential_offer_json, cb) => {
                trace!(target: "prover_command_executor", "StoreCredentialOffer command received");
                cb(self.store_credential_offer(wallet_handle, &credential_offer_json));
            }
            ProverCommand::GetCredentialOffers(wallet_handle, filter_json, cb) => {
                trace!(target: "prover_command_executor", "GetCredentialOffers command received");
                cb(self.get_credential_offers(wallet_handle, &filter_json));
            }
            ProverCommand::CreateMasterSecret(wallet_handle, master_secret_name, cb) => {
                trace!(target: "prover_command_executor", "CreateMasterSecret command received");
                cb(self.create_master_secret(wallet_handle, &master_secret_name));
            }
            ProverCommand::CreateAndStoreCredentialRequest(wallet_handle, prover_did, credential_offer_json,
                                                           credential_def_json, master_secret_name, cb) => {
                trace!(target: "prover_command_executor", "CreateAndStoreCredentialRequest command received");
                cb(self.create_and_store_credential_request(wallet_handle, &prover_did, &credential_offer_json,
                                                            &credential_def_json, &master_secret_name));
            }
            ProverCommand::StoreCredential(wallet_handle, id, credential_json, rev_reg_def_json, rev_reg_entry_json, cb) => {
                trace!(target: "prover_command_executor", "StoreCredential command received");
                cb(self.store_credential(wallet_handle,
                                         &id,
                                         &credential_json,
                                         rev_reg_def_json.as_ref().map(String::as_str),
                                         rev_reg_entry_json.as_ref().map(String::as_str)));
            }
            ProverCommand::GetCredentials(wallet_handle, filter_json, cb) => {
                trace!(target: "prover_command_executor", "GetCredentials command received");
                cb(self.get_credentials(wallet_handle, &filter_json));
            }
            ProverCommand::GetCredentialsForProofReq(wallet_handle, proof_req_json, cb) => {
                trace!(target: "prover_command_executor", "GetCredentialsForProofReq command received");
                cb(self.get_credentials_for_proof_req(wallet_handle, &proof_req_json));
            }
            ProverCommand::CreateProof(wallet_handle, proof_req_json, requested_credentials_json, schemas_json,
                                       master_secret_name, credential_defs_json, rev_reg_entries_json, cb) => {
                trace!(target: "prover_command_executor", "CreateProof command received");
                cb(self.create_proof(wallet_handle, &proof_req_json, &requested_credentials_json, &schemas_json,
                                     &master_secret_name, &credential_defs_json, &rev_reg_entries_json));
            }
            ProverCommand::CreateWitness(wallet_handle, tails_reader_handle, rev_reg_def_json, rev_reg_delta_json, rev_idx, cb) => {
                trace!(target: "prover_command_executor", "CreateWitness command received");
                cb(self.create_witness(wallet_handle, tails_reader_handle, &rev_reg_def_json, &rev_reg_delta_json, rev_idx));
            }
            ProverCommand::UpdateWitness(wallet_handle, tails_reader_handle, witness_json, rev_reg_def_json, rev_reg_delta_json, rev_idx, cb) => {
                trace!(target: "prover_command_executor", "UpdateWitness command received");
                cb(self.update_witness(wallet_handle, tails_reader_handle, &witness_json, &rev_reg_def_json, &rev_reg_delta_json, rev_idx));
            }
            ProverCommand::StoreWitness(wallet_handle, id, witness_json, cb) => {
                trace!(target: "prover_command_executor", "StoreWitness command received");
                cb(self.store_witness(wallet_handle, &id, &witness_json));
            }
            ProverCommand::GetWitness(wallet_handle, id, cb) => {
                trace!(target: "prover_command_executor", "GetWitness command received");
                cb(self.get_witness(wallet_handle, &id));
            }
        };
    }

    fn store_credential_offer(&self,
                              wallet_handle: i32,
                              credential_offer_json: &str) -> Result<(), IndyError> {
        trace!("store_credential_offer >>> wallet_handle: {:?}, credential_offer_json: {:?}", wallet_handle, credential_offer_json);

        let credential_offer: CredentialOffer = CredentialOffer::from_json(credential_offer_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize CredentialOffer: {:?}", err)))?;

        self.crypto_service.validate_did(&credential_offer.issuer_did)?;

        let id = get_composite_id(&credential_offer.issuer_did, &credential_offer.schema_key);
        self.wallet_service.set(wallet_handle, &format!("credential_offer::{}", &id), &credential_offer_json)?;

        trace!("store_credential_offer <<<");

        Ok(())
    }

    fn get_credential_offers(&self,
                             wallet_handle: i32,
                             filter_json: &str) -> Result<String, IndyError> {
        trace!("get_credential_offers >>> wallet_handle: {:?}, filter_json: {:?}", wallet_handle, filter_json);

        let credential_offer_jsons: Vec<(String, String)> = self.wallet_service.list(wallet_handle, &format!("credential_offer::"))?;

        let mut credential_offers: Vec<CredentialOffer> = Vec::new();
        for &(ref id, ref credential_offer_json) in credential_offer_jsons.iter() {
            credential_offers.push(CredentialOffer::from_json(credential_offer_json)
                .map_err(|err| CommonError::InvalidState(format!("Cannot deserialize CredentialOffer: {:?}", err)))?);
        }

        let filter: Filter = Filter::from_json(filter_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize Filter: {:?}", err)))?;

        credential_offers.retain(|credential_offer|
            self.anoncreds_service.prover.satisfy_restriction(credential_offer, &filter));

        let credential_offers_json = serde_json::to_string(&credential_offers)
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize list of CredentialOffer: {:?}", err)))?;

        trace!("get_credential_offers <<< credential_offers_json: {:?}", credential_offers_json);

        Ok(credential_offers_json)
    }

    fn create_master_secret(&self,
                            wallet_handle: i32,
                            master_secret_name: &str) -> Result<(), IndyError> {
        trace!("create_master_secret >>> wallet_handle: {:?}, master_secret_name: {:?}", wallet_handle, master_secret_name);

        if let Ok(_) = self.wallet_service.get(wallet_handle, &format!("master_secret::{}", master_secret_name)) {
            return Err(IndyError::AnoncredsError(
                AnoncredsError::MasterSecretDuplicateNameError(format!("MasterSecret already exists {}", master_secret_name))));
        };

        let master_secret = self.anoncreds_service.prover.new_master_secret()?;
        let master_secret_json =
            self.wallet_service.set_object(wallet_handle, &format!("master_secret::{}", master_secret_name), &master_secret, "MasterSecret")?;

        trace!("create_master_secret <<<");

        Ok(())
    }

    fn create_and_store_credential_request(&self,
                                           wallet_handle: i32,
                                           prover_did: &str,
                                           credential_offer_json: &str,
                                           credential_def_json: &str,
                                           master_secret_name: &str) -> Result<String, IndyError> {
        trace!("create_and_store_credential_request >>> wallet_handle: {:?}, prover_did: {:?}, credential_offer_json: {:?}, credential_def_json: {:?}, \
               master_secret_name: {:?}", wallet_handle, prover_did, credential_offer_json, credential_def_json, master_secret_name);

        self.crypto_service.validate_did(&prover_did)?;

        let master_secret: MasterSecret =
            self.wallet_service.get_object(wallet_handle, &format!("master_secret::{}", &master_secret_name), "MasterSecret", &mut String::new())?;

        let credential_def: CredentialDefinition = CredentialDefinition::from_json(&credential_def_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize CredentialDefinition: {:?}", err)))?;

        let credential_offer: CredentialOffer = CredentialOffer::from_json(&credential_offer_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize CredentialOffer: {:?}", err)))?;

        if credential_def.issuer_did != credential_offer.issuer_did {
            return Err(IndyError::CommonError(CommonError::InvalidStructure(
                format!("CredentialOffer issuer_did {:?} does not correspond to CredentialDefinition issuer_did {:?}",
                        credential_offer.issuer_did, credential_def.issuer_did))));
        }

        let (credential_request, master_secret_blinding_data) =
            self.anoncreds_service.prover.new_credential_request(&credential_def, &master_secret, &credential_offer, prover_did)?;

        let nonce = credential_request.nonce.clone()
            .map_err(|err| CommonError::InvalidState(format!("Cannot deserialize Nonce: {:?}", err)))?;

        let credential_request_metadata = CredentialRequestMetadata {
            master_secret_blinding_data,
            nonce,
            master_secret_name: master_secret_name.to_string()
        };

        let id = get_composite_id(&credential_offer.issuer_did, &credential_offer.schema_key);

        self.wallet_service.set_object(wallet_handle, &format!("credential_request_metadata::{}", id), &credential_request_metadata, "CredentialRequestMetadata")?;

        self.wallet_service.set(wallet_handle, &format!("credential_definition::{}", id), &credential_def_json)?;

        let credential_request_json = credential_request.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize CredentialRequest: {:?}", err)))?;

        trace!("create_and_store_credential_request <<< credential_request_json: {:?}", credential_request_json);

        Ok(credential_request_json)
    }

    fn store_credential(&self,
                        wallet_handle: i32,
                        id: &str,
                        credential_json: &str,
                        rev_reg_def_json: Option<&str>,
                        rev_reg_entry_json: Option<&str>) -> Result<(), IndyError> {
        trace!("store_credential >>> wallet_handle: {:?}, credential_json: {:?}, rev_reg_def_json: {:?}, rev_reg_entry_json: {:?}",
               wallet_handle, credential_json, rev_reg_def_json, rev_reg_entry_json);

        let mut credential: Credential = Credential::from_json(&credential_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize Credential: {:?}", err)))?;

        let rev_reg_def = match rev_reg_def_json {
            Some(r_reg_def_json) =>
                Some(RevocationRegistryDefinition::from_json(&r_reg_def_json)
                    .map_err(|err| CommonError::InvalidState(format!("Cannot deserialize RevocationRegistryDefinition: {:?}", err)))?),
            None => None
        };

        let rev_reg = match rev_reg_entry_json {
            Some(r_reg_entry_json) =>
                Some(RevocationRegistry::from_json(&r_reg_entry_json)
                    .map_err(|err| CommonError::InvalidState(format!("Cannot deserialize RevocationRegistry: {:?}", err)))?),
            None => None
        };

        let key_id = get_composite_id(&credential.issuer_did, &credential.schema_key);

        let credential_request_metadata: CredentialRequestMetadata =
            self.wallet_service.get_object(wallet_handle, &format!("credential_request_metadata::{}", key_id), "CredentialRequestMetadata", &mut String::new())?;

        let master_secret: MasterSecret =
            self.wallet_service.get_object(wallet_handle, &format!("master_secret::{}", &credential_request_metadata.master_secret_name), "MasterSecret", &mut String::new())?;

        let credential_def: CredentialDefinition =
            self.wallet_service.get_object(wallet_handle, &format!("credential_definition::{}", key_id), "CredentialDefinition", &mut String::new())?;

        let witness = self.wallet_service.get_opt_object::<Witness>(wallet_handle, &format!("witness::{}", &id), "Witness", &mut String::new())?;

        self.anoncreds_service.prover.process_credential(&mut credential,
                                                         &credential_request_metadata,
                                                         &master_secret,
                                                         &credential_def,
                                                         rev_reg_def.as_ref(),
                                                         rev_reg.as_ref(),
                                                         witness.as_ref())?;

        self.wallet_service.set_object(wallet_handle, &format!("credential::{}", &id), &credential, "Credential")?;

        if let Some(r_reg_def_json) = rev_reg_def_json {
            self.wallet_service.set(wallet_handle, &format!("revocation_registry_definition::{}", id), &r_reg_def_json)?;
        }

        if let Some(r_reg_entry_json) = rev_reg_entry_json {
            self.wallet_service.set(wallet_handle, &format!("revocation_registry::{}", id), &r_reg_entry_json)?;
        }

        trace!("store_credential <<<");

        Ok(())
    }

    fn get_credentials(&self,
                       wallet_handle: i32,
                       filter_json: &str) -> Result<String, IndyError> {
        trace!("get_credentials >>> wallet_handle: {:?}, filter_json: {:?}", wallet_handle, filter_json);

        let mut credentials_info: Vec<CredentialInfo> = self.get_credentials_info(wallet_handle)?;

        let filter: Filter = Filter::from_json(filter_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize Filter: {:?}", err)))?;

        credentials_info.retain(move |credential_info|
            self.anoncreds_service.prover.satisfy_restriction(credential_info, &filter));

        let credentials_info_json = serde_json::to_string(&credentials_info)
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize list of CredentialInfo: {:?}", err)))?;

        trace!("get_credentials <<< credentials_info_json: {:?}", credentials_info_json);

        Ok(credentials_info_json)
    }

    fn get_credentials_info(&self,
                            wallet_handle: i32) -> Result<Vec<CredentialInfo>, IndyError> {
        trace!("get_credentials_info >>> wallet_handle: {:?}", wallet_handle);

        let credentials: Vec<(String, String)> = self.wallet_service.list(wallet_handle, &format!("credential::"))?;

        let mut credentials_info: Vec<CredentialInfo> = Vec::new();

        for &(ref referent, ref credential) in credentials.iter() {
            let credential: Credential = Credential::from_json(credential)
                .map_err(|err| CommonError::InvalidState(format!("Cannot deserialize Credential: {:?}", err)))?;

            let mut credential_values: HashMap<String, String> = HashMap::new();
            for (attr, values) in credential.values {
                credential_values.insert(attr.clone(), values[0].clone());
            }

            credentials_info.push(
                CredentialInfo {
                    referent: referent.replace("credential::", ""),
                    attrs: credential_values,
                    schema_key: credential.schema_key.clone(),
                    issuer_did: credential.issuer_did.clone(),
                    revoc_reg_seq_no: credential.rev_reg_seq_no.clone()
                });
        }

        trace!("get_credentials_info <<< credentials_info: {:?}", credentials_info);

        Ok(credentials_info)
    }

    fn get_credentials_for_proof_req(&self,
                                     wallet_handle: i32,
                                     proof_req_json: &str, ) -> Result<String, IndyError> {
        trace!("get_credentials_for_proof_req >>> wallet_handle: {:?}, proof_req_json: {:?}", wallet_handle, proof_req_json);

        let proof_request: ProofRequest = ProofRequest::from_json(proof_req_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize ProofRequest: {:?}", err)))?;

        let credentials: Vec<CredentialInfo> = self.get_credentials_info(wallet_handle)?;

        let credentials_for_proof_request = self.anoncreds_service.prover.get_credentials_for_proof_req(&proof_request, &credentials)?;
        let credentials_for_proof_request_json = credentials_for_proof_request.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize CredentialsForProofRequest: {:?}", err)))?;

        trace!("get_credentials_for_proof_req <<< credentials_for_proof_request_json: {:?}", credentials_for_proof_request_json);

        Ok(credentials_for_proof_request_json)
    }

    fn create_witness(&self,
                      wallet_handle: i32,
                      tails_reader_handle: i32,
                      rev_reg_def: &str,
                      rev_reg_delta_json: &str,
                      rev_idx: u32) -> Result<String, IndyError> {
        trace!("create_witness >>> wallet_handle: {:?}, tails_reader_handle: {:?}, rev_reg_def: {:?}, rev_reg_delta_json: {:?}, rev_idx: {:?}",
               wallet_handle, tails_reader_handle, rev_reg_def, rev_reg_delta_json, rev_idx);

        let revocation_registry_definition: RevocationRegistryDefinition = RevocationRegistryDefinition::from_json(rev_reg_def)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize RevocationRegistryDefinition: {:?}", err)))?;

        let rev_reg_delta: RevocationRegistryDelta = RevocationRegistryDelta::from_json(rev_reg_delta_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize RevocationRegistryDelta: {:?}", err)))?;

        let sdk_tails_accessor = SDKTailsAccessor::new(self.blob_storage_service.clone(), tails_reader_handle);

        let witness = Witness::new(rev_idx, revocation_registry_definition.max_cred_num, &rev_reg_delta, &sdk_tails_accessor)
            .map_err(|err| IndyError::CommonError(CommonError::from(err)))?;

        let witness_json = witness.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize Witness: {:?}", err)))?;

        trace!("create_witness <<< witness_json: {:?}", witness_json);

        Ok(witness_json)
    }

    fn update_witness(&self,
                      wallet_handle: i32,
                      tails_reader_handle: i32,
                      witness_json: &str,
                      rev_reg_def_json: &str,
                      rev_reg_delta_json: &str,
                      rev_idx: u32) -> Result<String, IndyError> {
        trace!("update_witness >>> wallet_handle: {:?}, tails_reader_handle: {:?}, rev_reg_def_json: {:?}, rev_reg_delta_json: {:?}, rev_idx: {:?}",
               wallet_handle, tails_reader_handle, rev_reg_def_json, rev_reg_delta_json, rev_idx);

        let mut witness: Witness = Witness::from_json(witness_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize Witness: {:?}", err)))?;

        let revocation_registry_definition: RevocationRegistryDefinition = RevocationRegistryDefinition::from_json(rev_reg_def_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize RevocationRegistryDefinition: {:?}", err)))?;

        let rev_reg_delta: RevocationRegistryDelta = RevocationRegistryDelta::from_json(rev_reg_delta_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize RevocationRegistryDelta: {:?}", err)))?;

        let sdk_tails_accessor = SDKTailsAccessor::new(self.blob_storage_service.clone(), tails_reader_handle);

        witness.update(rev_idx, revocation_registry_definition.max_cred_num, &rev_reg_delta, &sdk_tails_accessor)
            .map_err(|err| IndyError::CommonError(CommonError::from(err)))?;

        let witness_json = witness.to_json()
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize Witness: {:?}", err)))?;

        trace!("update_witness <<< witness_json: {:?}", witness_json);

        Ok(witness_json)
    }

    fn store_witness(&self,
                     wallet_handle: i32,
                     id: &str,
                     witness_json: &str) -> Result<(), IndyError> {
        trace!("store_witness >>> wallet_handle: {:?}, id: {:?}, witness_json: {:?}", wallet_handle, id, witness_json);

        self.wallet_service.set(wallet_handle, &format!("witness::{}", &id), witness_json)?;

        trace!("store_witness <<< witness_json: {:?}", witness_json);

        Ok(())
    }

    fn get_witness(&self,
                   wallet_handle: i32,
                   id: &str) -> Result<String, IndyError> {
        trace!("store_witness >>> wallet_handle: {:?}, id: {:?}", wallet_handle, id);

        let witness_json = self.wallet_service.get(wallet_handle, &format!("witness::{}", id))?;

        trace!("store_witness <<< witness_json: {:?}", witness_json);

        Ok(witness_json)
    }

    fn create_proof(&self,
                    wallet_handle: i32,
                    proof_req_json: &str,
                    requested_credentials_json: &str,
                    schemas_json: &str,
                    master_secret_name: &str,
                    credential_defs_json: &str,
                    rev_reg_entries_json: &str) -> Result<String, IndyError> {
        let proof_req: ProofRequest = ProofRequest::from_json(proof_req_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize ProofRequest: {:?}", err)))?;

        let schemas: HashMap<String, Schema> = serde_json::from_str(schemas_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize list of Schema: {:?}", err)))?;

        let credential_defs: HashMap<String, CredentialDefinition> = serde_json::from_str(credential_defs_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize list of CredentialDefinition: {:?}", err)))?;

        let rev_regs: HashMap<String, RevocationRegistry> = serde_json::from_str(rev_reg_entries_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize list of RevocationRegistryEntry: {:?}", err)))?;

        let requested_credentials: RequestedCredentials = RequestedCredentials::from_json(requested_credentials_json)
            .map_err(|err| CommonError::InvalidStructure(format!("Cannot deserialize RequestedCredentials: {:?}", err)))?;

        if schemas.keys().collect::<HashSet<&String>>() != credential_defs.keys().collect::<HashSet<&String>>() {
            return Err(IndyError::CommonError(CommonError::InvalidStructure(
                format!("CredentialDefinitions {:?} do not correspond to Schemas {:?}", schemas.keys(), credential_defs.keys()))));
        }

        let master_secret: MasterSecret =
            self.wallet_service.get_object(wallet_handle, &format!("master_secret::{}", master_secret_name), "MasterSecret", &mut String::new())?;

        let mut credentials: HashMap<String, Credential> = HashMap::new();
        let mut witnesses: HashMap<String, Witness> = HashMap::new();

        for key_id in credential_defs.keys() {
            let credential: Credential = self.wallet_service.get_object(wallet_handle, &format!("credential::{}", &key_id), "Credential", &mut String::new())?;
            credentials.insert(key_id.clone(), credential);

            let witness = self.wallet_service.get_opt_object::<Witness>(wallet_handle, &format!("witness::{}", &key_id), "Witness", &mut String::new())?;
            witness.map(|witness| witnesses.insert(key_id.clone(), witness));
        }

        let proof_credentials = self.anoncreds_service.prover.create_proof(&credentials,
                                                                           &proof_req,
                                                                           &requested_credentials,
                                                                           &master_secret,
                                                                           &schemas,
                                                                           &credential_defs,
                                                                           &rev_regs,
                                                                           &witnesses)?;

        let proof_credentials_json = FullProof::to_json(&proof_credentials)
            .map_err(|err| CommonError::InvalidState(format!("Cannot serialize Proof: {:?}", err)))?;

        Ok(proof_credentials_json)
    }
}