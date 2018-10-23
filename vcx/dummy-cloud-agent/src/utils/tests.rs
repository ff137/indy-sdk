use domain::a2a::*;
use domain::config::*;
use futures::*;
use indy::{did, IndyError, wallet};
use tokio_core::reactor::Core;

pub const EDGE_AGENT_WALLET_ID: &'static str = "edge_agent_wallet_id";
pub const EDGE_AGENT_WALLET_CONFIG: &'static str = "{\"id\": \"edge_agent_wallet_id\"}";
pub const EDGE_AGENT_WALLET_PASSPHRASE: &'static str = "edge_agent_wallet_passphrase";
pub const EDGE_AGENT_WALLET_CREDENTIALS: &'static str = "{\"key\": \"edge_agent_wallet_passphrase\"}";
pub const EDGE_AGENT_DID: &'static str = "NcYxiDXkpYi6ov5FcYDi1e";
pub const EDGE_AGENT_DID_INFO: &'static str = "{\"did\": \"NcYxiDXkpYi6ov5FcYDi1e\", \"seed\": \"0000000000000000000000000000Edge\"}";
pub const EDGE_AGENT_DID_VERKEY: &'static str = "B4aUxMQdPFkwBtcNUgs4fAbJhLSbXjQmrXByzL6gfDEq";

pub const FORWARD_AGENT_WALLET_ID: &'static str = "forward_agent_wallet_id";
pub const FORWARD_AGENT_WALLET_CONFIG: &'static str = "{\"id\": \"forward_agent_wallet_id\"}";
pub const FORWARD_AGENT_WALLET_PASSPHRASE: &'static str = "forward_agent_wallet_passphrase";
pub const FORWARD_AGENT_WALLET_CREDENTIALS: &'static str = "{\"key\": \"forward_agent_wallet_passphrase\"}";
pub const FORWARD_AGENT_DID: &'static str = "VsKV7grR1BUE29mG2Fm2kX";
pub const FORWARD_AGENT_DID_SEED: &'static str = "0000000000000000000000000Forward";
pub const FORWARD_AGENT_DID_INFO: &'static str = "{\"did\": \"VsKV7grR1BUE29mG2Fm2kX\", \"seed\": \"0000000000000000000000000Forward\"}";
pub const FORWARD_AGENT_DID_VERKEY: &'static str = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";

pub fn forward_agent_config() -> ForwardAgentConfig {
    ForwardAgentConfig {
        wallet_id: FORWARD_AGENT_WALLET_ID.into(),
        wallet_passphrase: FORWARD_AGENT_WALLET_PASSPHRASE.into(),
        did: FORWARD_AGENT_DID.into(),
        did_seed: Some(FORWARD_AGENT_DID_SEED.into()),
    }
}

pub fn wallet_storage_config() -> WalletStorageConfig {
    WalletStorageConfig {
        xtype: None,
        config: None,
        credentials: None,
    }
}

pub fn edge_agent_wallet_setup() -> i32 {
    let mut core = Core::new().unwrap();

    core.run(
        wallet::create_wallet(EDGE_AGENT_WALLET_CONFIG, EDGE_AGENT_WALLET_CREDENTIALS)
            .then(|res| {
                match res {
                    Err(IndyError::WalletAlreadyExistsError) => Ok(()),
                    r => r
                }
            })).unwrap();

    let wallet_handle = core.run(
        wallet::open_wallet(EDGE_AGENT_WALLET_CONFIG, EDGE_AGENT_WALLET_CREDENTIALS)
    ).unwrap();

    core.run(
        did::create_and_store_my_did(wallet_handle, EDGE_AGENT_DID_INFO)
            .then(|res| match res {
                Ok(_) => Ok(()),
                Err(IndyError::DidAlreadyExistsError) => Ok(()), // Already exists
                Err(err) => Err(err),
            })
    ).unwrap();

    wallet_handle
}

pub fn forward_agent_wallet_setup() -> i32 {
    let mut core = Core::new().unwrap();

    core.run(
        wallet::create_wallet(FORWARD_AGENT_WALLET_CONFIG, FORWARD_AGENT_WALLET_CREDENTIALS)
            .then(|res| {
                match res {
                    Err(IndyError::WalletAlreadyExistsError) => Ok(()),
                    r => r
                }
            })).unwrap();

    let wallet_handle = core.run(
        wallet::open_wallet(FORWARD_AGENT_WALLET_CONFIG, FORWARD_AGENT_WALLET_CREDENTIALS)
    ).unwrap();

    core.run(
        did::create_and_store_my_did(wallet_handle, FORWARD_AGENT_DID_INFO)
            .then(|res| match res {
                Ok(_) => Ok(()),
                Err(IndyError::DidAlreadyExistsError) => Ok(()), // Already exists
                Err(err) => Err(err),
            })
    ).unwrap();

    wallet_handle
}

pub fn compose_connect(wallet_handle: i32) -> Vec<u8> {
    let mut core = Core::new().unwrap();

    let msgs = [A2AMessage::Connect(
        Connect {
            from_did: EDGE_AGENT_DID.into(),
            from_did_verkey: EDGE_AGENT_DID_VERKEY.into(),
        })];

    let msg = core.run(
        A2AMessage::bundle_authcrypted(wallet_handle,
                                       EDGE_AGENT_DID_VERKEY,
                                       FORWARD_AGENT_DID_VERKEY,
                                       &msgs)
    ).unwrap();

    compose_forward(FORWARD_AGENT_DID, FORWARD_AGENT_DID_VERKEY, msg)
}

pub fn compose_forward(recipient_did: &str, recipient_vk: &str, msg: Vec<u8>) -> Vec<u8> {
    let mut core = Core::new().unwrap();

    let msgs = [A2AMessage::Forward(
        Forward {
            fwd: recipient_did.into(),
            msg,
        })];

    core.run(
        A2AMessage::bundle_anoncrypted(recipient_vk, &msgs)
    ).unwrap()
}