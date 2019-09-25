pub mod issuer;
pub mod prover;
pub mod verifier;
mod tails;

use serde_json;

use commands::anoncreds::issuer::{IssuerCommand, IssuerCommandExecutor};
use commands::anoncreds::prover::{ProverCommand, ProverCommandExecutor};
use commands::anoncreds::verifier::{VerifierCommand, VerifierCommandExecutor};

use services::anoncreds::AnoncredsService;
use services::blob_storage::BlobStorageService;
use services::pool::PoolService;
use services::wallet::WalletService;
use services::crypto::CryptoService;

use domain::crypto::did::DidValue;
use domain::anoncreds::schema::SchemaId;
use domain::anoncreds::credential_definition::CredentialDefinitionId;
use domain::anoncreds::revocation_registry_definition::RevocationRegistryId;
use domain::anoncreds::credential_offer::CredentialOffer;

use errors::prelude::*;

use std::rc::Rc;

pub enum AnoncredsCommand {
    Issuer(IssuerCommand),
    Prover(ProverCommand),
    Verifier(VerifierCommand),
    Disqualify(
        String, // entity
        Box<dyn Fn(IndyResult<String>) + Send>)
}

pub struct AnoncredsCommandExecutor {
    issuer_command_cxecutor: IssuerCommandExecutor,
    prover_command_cxecutor: ProverCommandExecutor,
    verifier_command_cxecutor: VerifierCommandExecutor
}

impl AnoncredsCommandExecutor {
    pub fn new(anoncreds_service: Rc<AnoncredsService>,
               blob_storage_service: Rc<BlobStorageService>,
               pool_service: Rc<PoolService>,
               wallet_service: Rc<WalletService>,
               crypto_service: Rc<CryptoService>) -> AnoncredsCommandExecutor {
        AnoncredsCommandExecutor {
            issuer_command_cxecutor: IssuerCommandExecutor::new(
                anoncreds_service.clone(), pool_service.clone(),
                blob_storage_service.clone(), wallet_service.clone(), crypto_service.clone()),
            prover_command_cxecutor: ProverCommandExecutor::new(
                anoncreds_service.clone(), wallet_service.clone(), crypto_service.clone(), blob_storage_service.clone()),
            verifier_command_cxecutor: VerifierCommandExecutor::new(
                anoncreds_service.clone()),
        }
    }

    pub fn execute(&self, command: AnoncredsCommand) {
        match command {
            AnoncredsCommand::Issuer(cmd) => {
                debug!(target: "anoncreds_command_executor", "Issuer command received");
                self.issuer_command_cxecutor.execute(cmd);
            }
            AnoncredsCommand::Prover(cmd) => {
                debug!(target: "anoncreds_command_executor", "Prover command received");
                self.prover_command_cxecutor.execute(cmd);
            }
            AnoncredsCommand::Verifier(cmd) => {
                debug!(target: "anoncreds_command_executor", "Verifier command received");
                self.verifier_command_cxecutor.execute(cmd);
            }
            AnoncredsCommand::Disqualify(entity, cb) => {
                debug!("Disqualify command received");
                cb(self.disqualify(entity));
            }
        };
    }

    fn disqualify(&self,
                  entity: String) -> IndyResult<String> {
        info!("disqualify >>> entity: {:?}", entity);

        if entity.starts_with(DidValue::PREFIX) {
            return Ok(DidValue(entity).disqualify().0);
        }

        if entity.starts_with(SchemaId::PREFIX) {
            return Ok(SchemaId(entity).disqualify().0);
        }

        if entity.starts_with(CredentialDefinitionId::PREFIX) {
            return Ok(CredentialDefinitionId(entity).disqualify().0);
        }

        if entity.starts_with(RevocationRegistryId::PREFIX) {
            return Ok(RevocationRegistryId(entity).disqualify().0);
        }

        if let Ok(cred_offer) = ::serde_json::from_str::<CredentialOffer>(&entity) {
            let cred_offer = cred_offer.disqualify();
            return serde_json::to_string(&cred_offer)
                .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidState, format!("Cannot serialize Credential Offer: {:?}", err)));
        }

        Err(IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Cannot disqualify {:?}: unsupported type", entity)))
    }
}
