use command_executor::{Command, CommandContext, CommandMetadata, CommandParams, CommandGroup, CommandGroupMetadata};
use commands::*;

use libindy::ErrorCode;
use libindy::ledger::Ledger;

use serde_json::Value as JSONValue;
use serde_json::Map as JSONMap;

use std::collections::HashSet;
use std::fmt;

pub mod group {
    use super::*;

    command_group!(CommandGroupMetadata::new("ledger", "Ledger management commands"));
}

pub mod nym_command {
    use super::*;

    command!(CommandMetadata::build("nym", "Add NYM to Ledger.")
                .add_param("did", false, "DID of new identity")
                .add_param("verkey", true, "Verification key of new identity")
                .add_param("alias", true, "Alias of new identity")
                .add_param("role", true, "Role of new identity. One of: STEWARD, TRUSTEE, TRUST_ANCHOR, TGB")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let submitter_did = ensure_active_did(&ctx)?;
        let pool_handle = ensure_connected_pool_handle(&ctx)?;
        let wallet_handle = ensure_opened_wallet_handle(&ctx)?;

        let target_did = get_str_param("did", params).map_err(error_err!())?;
        let verkey = get_opt_str_param("verkey", params).map_err(error_err!())?;
        let alias = get_opt_str_param("alias", params).map_err(error_err!())?;
        let role = get_opt_str_param("role", params).map_err(error_err!())?;

        let res = Ledger::build_nym_request(&submitter_did, target_did, verkey, alias, role)
            .and_then(|request| Ledger::sign_and_submit_request(pool_handle, wallet_handle, &submitter_did, &request));

        let res = match res {
            Ok(_) => Ok(println_succ!("NYM {{\"did\":\"{}\", \"verkey\":\"{:?}\", \"alias\":\"{:?}\", \"role\":\"{:?}\"}} has been added to Ledger",
                                      target_did, verkey, alias, role)),
            Err(err) => handle_send_command_error(err, &submitter_did, pool_handle, wallet_handle)
        };

        trace!("execute << {:?}", res);
        res
    }
}

pub mod get_nym_command {
    use super::*;

    command!(CommandMetadata::build("get-nym", "Get NYM from Ledger.")
                .add_param("did", false, "DID of identity presented in Ledger")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let submitter_did = ensure_active_did(&ctx)?;
        let pool_handle = ensure_connected_pool_handle(&ctx)?;

        let target_did = get_str_param("did", params).map_err(error_err!())?;

        let res = Ledger::build_get_nym_request(&submitter_did, target_did)
            .and_then(|request| Ledger::submit_request(pool_handle, &request));

        let response = match res {
            Ok(response) => Ok(response),
            Err(err) => handle_get_command_error(err),
        }?;

        let nym = serde_json::from_str::<Reply<String>>(&response)
            .and_then(|response| serde_json::from_str::<NymData>(&response.result.data));

        let res = match nym {
            Ok(nym) => Ok(println_succ!("Following NYM has been received: {}", nym)),
            Err(_) => Err(println_err!("NYM not found"))
        };

        trace!("execute << {:?}", res);
        res
    }
}

pub mod attrib_command {
    use super::*;

    command!(CommandMetadata::build("attrib", "Add Attribute to exists NYM.")
                .add_param("did", false, "DID of identity presented in Ledger")
                .add_param("hash", true, "Hash of attribute data")
                .add_param("raw", true, "JSON representation of attribute data")
                .add_param("enc", true, "Encrypted attribute data")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let submitter_did = ensure_active_did(&ctx)?;
        let pool_handle = ensure_connected_pool_handle(&ctx)?;
        let wallet_handle = ensure_opened_wallet_handle(&ctx)?;

        let target_did = get_str_param("did", params).map_err(error_err!())?;
        let hash = get_opt_str_param("hash", params).map_err(error_err!())?;
        let raw = get_opt_str_param("raw", params).map_err(error_err!())?;
        let enc = get_opt_str_param("enc", params).map_err(error_err!())?;


        let res = Ledger::build_attrib_request(&submitter_did, target_did, hash, raw, enc)
            .and_then(|request| Ledger::sign_and_submit_request(pool_handle, wallet_handle, &submitter_did, &request));

        let attribute = raw.unwrap_or(hash.unwrap_or(enc.unwrap_or("")));

        let res = match res {
            Ok(_) => Ok(println_succ!("Attribute \"{}\" has been added to Ledger", attribute)),
            Err(err) => handle_send_command_error(err, &submitter_did, pool_handle, wallet_handle)
        };

        trace!("execute << {:?}", res);
        res
    }
}

pub mod get_attrib_command {
    use super::*;

    command!(CommandMetadata::build("get-attrib", "Get ATTRIB from Ledger.")
                .add_param("did", false, "DID of identity presented in Ledger")
                .add_param("attr", false, "Name of attribute")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let submitter_did = ensure_active_did(&ctx)?;
        let pool_handle = ensure_connected_pool_handle(&ctx)?;

        let target_did = get_str_param("did", params).map_err(error_err!())?;
        let attr = get_str_param("attr", params).map_err(error_err!())?;

        let res = Ledger::build_get_attrib_request(&submitter_did, target_did, attr)
            .and_then(|request| Ledger::submit_request(pool_handle, &request));

        let response = match res {
            Ok(response) => Ok(response),
            Err(err) => handle_get_command_error(err),
        }?;

        let attrib = serde_json::from_str::<Reply<String>>(&response)
            .and_then(|response| serde_json::from_str::<AttribData>(&response.result.data));

        let res = match attrib {
            Ok(nym) => Ok(println_succ!("Following Attribute has been received: {}", nym)),
            Err(_) => Err(println_err!("Attribute not found"))
        };

        trace!("execute << {:?}", res);
        res
    }
}

pub mod schema_command {
    use super::*;

    command!(CommandMetadata::build("schema", "Add Schema to Ledger.")
                .add_param("name", false, "Schema name")
                .add_param("version", false, "Schema version")
                .add_param("attr_names", false, "Schema attributes split by comma")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let submitter_did = ensure_active_did(&ctx)?;
        let pool_handle = ensure_connected_pool_handle(&ctx)?;
        let wallet_handle = ensure_opened_wallet_handle(&ctx)?;

        let name = get_str_param("name", params).map_err(error_err!())?;
        let version = get_str_param("version", params).map_err(error_err!())?;
        let attr_names = get_str_array_param("attr_names", params).map_err(error_err!())?;

        let schema_data = {
            let mut json = JSONMap::new();
            json.insert("name".to_string(), JSONValue::from(name));
            json.insert("version".to_string(), JSONValue::from(version));
            json.insert("attr_names".to_string(), JSONValue::from(attr_names));
            JSONValue::from(json).to_string()
        };

        let res = Ledger::build_schema_request(&submitter_did, &schema_data)
            .and_then(|request| Ledger::sign_and_submit_request(pool_handle, wallet_handle, &submitter_did, &request));

        let res = match res {
            Ok(_) => Ok(println_succ!("Schema {{name: \"{}\" version: \"{}\"}}  has been added to Ledger", name, version)),
            Err(err) => handle_send_command_error(err, &submitter_did, pool_handle, wallet_handle)
        };

        trace!("execute << {:?}", res);
        res
    }
}

pub mod get_schema_command {
    use super::*;

    command!(CommandMetadata::build("get-schema", "Get Schema from Ledger.")
                .add_param("did", false, "DID of identity presented in Ledger")
                .add_param("name", false, "Schema name")
                .add_param("version", false, "Schema version")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let submitter_did = ensure_active_did(&ctx)?;
        let pool_handle = ensure_connected_pool_handle(&ctx)?;

        let target_did = get_str_param("did", params).map_err(error_err!())?;
        let name = get_str_param("name", params).map_err(error_err!())?;
        let version = get_str_param("version", params).map_err(error_err!())?;

        let schema_data = {
            let mut json = JSONMap::new();
            json.insert("name".to_string(), JSONValue::from(name));
            json.insert("version".to_string(), JSONValue::from(version));
            JSONValue::from(json).to_string()
        };

        let res = Ledger::build_get_schema_request(&submitter_did, target_did, &schema_data)
            .and_then(|request| Ledger::submit_request(pool_handle, &request));

        let response = match res {
            Ok(response) => Ok(response),
            Err(err) => handle_get_command_error(err),
        }?;

        let res = match serde_json::from_str::<Reply<SchemaData>>(&response) {
            Ok(schema) => Ok(println_succ!("Following Schema has been received: \"{}\"", schema.result.data)),
            Err(_) => Err(println_err!("Schema not found"))
        };

        trace!("execute << {:?}", res);
        res
    }
}

pub mod claim_def_command {
    use super::*;

    command!(CommandMetadata::build("claim-def", "Add claim definition to Ledger.")
                .add_param("schema_no", false, "Sequence number of schema")
                .add_param("signature_type", false, "Signature type (only CL supported now)")
                .add_param("primary", false, "Primary key in json format")
                .add_param("revocation", true, "Revocation key in json format")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let submitter_did = ensure_active_did(&ctx)?;
        let pool_handle = ensure_connected_pool_handle(&ctx)?;
        let wallet_handle = ensure_opened_wallet_handle(&ctx)?;

        let xref = get_int_param::<i32>("schema_no", params).map_err(error_err!())?;
        let signature_type = get_str_param("signature_type", params).map_err(error_err!())?;
        let primary = get_object_param("primary", params).map_err(error_err!())?;
        let revocation = get_opt_str_param("revocation", params).map_err(error_err!())?;

        let claim_def_data = {
            let mut json = JSONMap::new();
            json.insert("primary".to_string(), primary);
            update_json_map_opt_key!(json, "revocation", revocation);
            JSONValue::from(json).to_string()
        };

        let res = Ledger::build_claim_def_txn(&submitter_did, xref, signature_type, &claim_def_data)
            .and_then(|request| Ledger::sign_and_submit_request(pool_handle, wallet_handle, &submitter_did, &request));

        let res = match res {
            Ok(_) => Ok(println_succ!("Claim def {{\"origin\":\"{}\", \"schema_seq_no\":{}, \"signature_type\":{}}} has been added to Ledger",
                                      submitter_did, xref, signature_type)),
            Err(err) => handle_send_command_error(err, &submitter_did, pool_handle, wallet_handle)
        };

        trace!("execute << {:?}", res);
        res
    }
}

pub mod get_claim_def_command {
    use super::*;

    command!(CommandMetadata::build("get-claim-def", "Add claim definition to Ledger.")
                .add_param("schema_no", false, "Sequence number of schema")
                .add_param("signature_type", false, "Signature type (only CL supported now)")
                .add_param("origin", false, "Claim definition owner DID")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let submitter_did = ensure_active_did(&ctx)?;
        let pool_handle = ensure_connected_pool_handle(&ctx)?;

        let xref = get_int_param::<i32>("schema_no", params).map_err(error_err!())?;
        let signature_type = get_str_param("signature_type", params).map_err(error_err!())?;
        let origin = get_str_param("origin", params).map_err(error_err!())?;

        let res = Ledger::build_get_claim_def_txn(&submitter_did, xref, signature_type, origin)
            .and_then(|request| Ledger::submit_request(pool_handle, &request));

        let response = match res {
            Ok(response) => Ok(response),
            Err(err) => handle_get_command_error(err),
        }?;

        let res = match serde_json::from_str::<Reply<ClaimDefData>>(&response) {
            Ok(claim_def) => Ok(println_succ!("Following ClaimDef has been received: \"{:?}\"", claim_def.result.data)),
            Err(_led) => Err(println_err!("Claim definition not found"))
        };

        trace!("execute << {:?}", res);
        res
    }
}

pub mod node_command {
    use super::*;

    command!(CommandMetadata::build("node", "Add Node to Ledger.")
                .add_param("target", false, "DID of new identity")
                .add_param("node_ip", false, "Node Ip")
                .add_param("node_port", false, "Node port")
                .add_param("client_ip", false, "Client Ip")
                .add_param("client_port", false, "Client port")
                .add_param("alias", false, "Node alias")
                .add_param("blskey", false, "Node BLS key")
                .add_param("services", true, "Node type [VALIDATOR, OBSERVER]")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let submitter_did = ensure_active_did(&ctx)?;
        let pool_handle = ensure_connected_pool_handle(&ctx)?;
        let wallet_handle = ensure_opened_wallet_handle(&ctx)?;

        let target_did = get_str_param("target", params).map_err(error_err!())?;
        let node_ip = get_opt_str_param("node_ip", params).map_err(error_err!())?;
        let node_port = get_opt_int_param::<i32>("node_port", params).map_err(error_err!())?;
        let client_ip = get_opt_str_param("client_ip", params).map_err(error_err!())?;
        let client_port = get_opt_int_param::<i32>("client_port", params).map_err(error_err!())?;
        let alias = get_opt_str_param("alias", params).map_err(error_err!())?;
        let blskey = get_opt_str_param("blskey", params).map_err(error_err!())?;
        let services = get_opt_str_array_param("services", params).map_err(error_err!())?;

        let node_data = {
            let mut json = JSONMap::new();
            update_json_map_opt_key!(json, "node_ip", node_ip);
            update_json_map_opt_key!(json, "node_port", node_port);
            update_json_map_opt_key!(json, "client_ip", client_ip);
            update_json_map_opt_key!(json, "client_port", client_port);
            update_json_map_opt_key!(json, "alias", alias);
            update_json_map_opt_key!(json, "blskey", blskey);
            update_json_map_opt_key!(json, "services", services);
            JSONValue::from(json).to_string()
        };

        let res = Ledger::build_node_request(&submitter_did, target_did, &node_data)
            .and_then(|request| Ledger::sign_and_submit_request(pool_handle, wallet_handle, &submitter_did, &request));

        let res = match res {
            Ok(_) => Ok(println_succ!("Node \"{}\" has been added to Ledger", node_data)),
            Err(err) => handle_send_command_error(err, &submitter_did, pool_handle, wallet_handle)
        };

        trace!("execute << {:?}", res);
        res
    }
}

pub mod custom_command {
    use super::*;

    command!(CommandMetadata::build("custom", "Send custom transaction to Ledger.")
                .add_main_param("txn", "Transaction json")
                .add_param("sign", true, "Is signature required")
                .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?} params {:?}", ctx, params);

        let pool_handle = ensure_connected_pool_handle(&ctx)?;

        let txn = get_str_param("txn", params).map_err(error_err!())?;
        let sign = get_opt_bool_param("sign", params).map_err(error_err!())?.unwrap_or(false);

        let res = if sign {
            let submitter_did = ensure_active_did(&ctx)?;
            let wallet_handle = ensure_opened_wallet_handle(&ctx)?;

            Ledger::sign_and_submit_request(pool_handle, wallet_handle, &submitter_did, txn)
        } else {
            Ledger::submit_request(pool_handle, txn)
        };

        let res = match res {
            Ok(response) => Ok(println_succ!("Response: {}", response)),
            Err(ErrorCode::LedgerInvalidTransaction) => Err(println_err!("Invalid transaction \"{}\"", txn)),
            Err(ErrorCode::WalletNotFoundError) => Err(println_err!("There is no active did")),
            Err(err) => Err(println_err!("Indy SDK error occurred {:?}", err)),
        };

        trace!("execute << {:?}", res);
        Ok(())
    }
}

fn handle_send_command_error(err: ErrorCode, submitter_did: &str, pool_handle: i32, wallet_handle: i32) -> Result<(), ()> {
    match err {
        ErrorCode::CommonInvalidStructure => Err(println_err!("Wrong command params")),
        ErrorCode::WalletNotFoundError => Err(println_err!("Submitter DID: \"{}\" not found", submitter_did)),
        ErrorCode::LedgerInvalidTransaction => Err(println_err!("Invalid transaction")),
        ErrorCode::WalletIncompatiblePoolError => Err(println_err!("Pool handle \"{}\" invalid for wallet handle \"{}\"", pool_handle, wallet_handle)),
        err => Err(println_err!("Indy SDK error occurred {:?}", err))
    }
}

fn handle_get_command_error(err: ErrorCode) -> Result<String, ()> {
    match err {
        ErrorCode::CommonInvalidStructure => Err(println_err!("Wrong command params")),
        ErrorCode::LedgerInvalidTransaction => Err(println_err!("Invalid transaction")),
        err => Err(println_err!("Indy SDK error occurred {:?}", err)),
    }
}

#[derive(Deserialize, Debug)]
pub struct Reply<T> {
    pub result: ReplyResult<T>,
}

#[derive(Deserialize, Debug)]
pub struct ReplyResult<T> {
    pub data: T
}

#[derive(Deserialize, Debug)]
pub struct NymData {
    pub identifier: Option<String>,
    pub dest: String,
    pub role: Option<String>,
    pub alias: Option<String>,
    pub verkey: Option<String>
}

impl fmt::Display for NymData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\nsubmitter:{} | did:{} | role:{} | alias:{} | verkey:{}",
               self.identifier.clone().unwrap_or("null".to_string()), self.dest,
               self.role.clone().unwrap_or("null".to_string()),
               self.alias.clone().unwrap_or("null".to_string()),
               self.verkey.clone().unwrap_or("null".to_string()))
    }
}

#[derive(Deserialize, Debug)]
pub struct AttribData {
    pub endpoint: Option<Endpoint>,
}

impl fmt::Display for AttribData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref endpoint) = self.endpoint {
            write!(f, "\n{:?}", endpoint)?;
        }
        write!(f, "")
    }
}

#[derive(Deserialize, Debug)]
pub struct Endpoint {
    pub ha: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SchemaData {
    pub attr_names: HashSet<String>,
    pub name: String,
    pub origin: Option<String>,
    pub version: String
}

impl fmt::Display for SchemaData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\nname:{} | version:{} | attr_names:{:?} | origin:{}",
               self.name, self.version, self.attr_names, self.origin.clone().unwrap_or("null".to_string()))
    }
}

#[derive(Deserialize, Debug)]
pub struct ClaimDefData {
    pub primary: serde_json::Value,
    pub revocation: Option<serde_json::Value>,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use commands::wallet::tests::{create_and_open_wallet, close_and_delete_wallet};
    use commands::pool::tests::{create_and_connect_pool, disconnect_and_delete_pool};
    use commands::did::tests::{new_did, use_did, SEED_TRUSTEE, DID_TRUSTEE};
    use libindy::ledger::Ledger;

    pub const MY1_SEED: &'static str = "00000000000000000000000000000My1";
    pub const DID_MY1: &'static str = "VsKV7grR1BUE29mG2Fm2kX";
    pub const VERKEY_MY1: &'static str = "GjZWsBLgZCR18aL468JAT7w9CZRiBnpxUPPgyQxh4voa";
    
    mod nym {
        use super::*;

        #[test]
        pub fn nym_works() {
            let ctx = CommandContext::new();

            create_and_open_wallet(&ctx);
            create_and_connect_pool(&ctx);

            new_did(&ctx, SEED_TRUSTEE);
            use_did(&ctx, DID_TRUSTEE);
            {
                let cmd = nym_command::new();
                let mut params = CommandParams::new();
                params.insert("did", DID_MY1.to_string());
                params.insert("verkey", VERKEY_MY1.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            _ensure_nym_added(&ctx);
            close_and_delete_wallet(&ctx);
            disconnect_and_delete_pool(&ctx);
        }
    }

    mod get_nym {
        use super::*;

        #[test]
        pub fn get_nym_works() {
            let ctx = CommandContext::new();

            create_and_open_wallet(&ctx);
            create_and_connect_pool(&ctx);

            new_did(&ctx, SEED_TRUSTEE);
            use_did(&ctx, DID_TRUSTEE);
            send_nym_my1(&ctx);
            {
                let cmd = get_nym_command::new();
                let mut params = CommandParams::new();
                params.insert("did", DID_MY1.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            close_and_delete_wallet(&ctx);
            disconnect_and_delete_pool(&ctx);
        }
    }

    mod attrib {
        use super::*;

        #[test]
        pub fn attrib_works() {
            let ctx = CommandContext::new();

            create_and_open_wallet(&ctx);
            create_and_connect_pool(&ctx);

            new_did(&ctx, SEED_TRUSTEE);
            new_did(&ctx, MY1_SEED);
            use_did(&ctx, DID_TRUSTEE);
            send_nym_my1(&ctx);
            use_did(&ctx, DID_MY1);
            {
                let cmd = attrib_command::new();
                let mut params = CommandParams::new();
                params.insert("did", DID_MY1.to_string());
                params.insert("raw", r#"{"endpoint":{"ha":"127.0.0.1:5555"}}"#.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            _ensure_attrib_added(&ctx);
            close_and_delete_wallet(&ctx);
            disconnect_and_delete_pool(&ctx);
        }
    }

    mod get_attrib {
        use super::*;

        #[test]
        pub fn get_attrib_works() {
            let ctx = CommandContext::new();

            create_and_open_wallet(&ctx);
            create_and_connect_pool(&ctx);

            new_did(&ctx, SEED_TRUSTEE);
            new_did(&ctx, MY1_SEED);
            use_did(&ctx, DID_TRUSTEE);
            send_nym_my1(&ctx);
            use_did(&ctx, DID_MY1);
            {
                let cmd = attrib_command::new();
                let mut params = CommandParams::new();
                params.insert("did", DID_MY1.to_string());
                params.insert("raw", r#"{"endpoint":{"ha":"127.0.0.1:5555"}}"#.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            {
                let cmd = get_attrib_command::new();
                let mut params = CommandParams::new();
                params.insert("did", DID_MY1.to_string());
                params.insert("attr", "endpoint".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            close_and_delete_wallet(&ctx);
            disconnect_and_delete_pool(&ctx);
        }
    }

    mod schema {
        use super::*;

        #[test]
        pub fn schema_works() {
            let ctx = CommandContext::new();

            create_and_open_wallet(&ctx);
            create_and_connect_pool(&ctx);

            new_did(&ctx, SEED_TRUSTEE);
            use_did(&ctx, DID_TRUSTEE);
            {
                let cmd = schema_command::new();
                let mut params = CommandParams::new();
                params.insert("name", "schema1".to_string());
                params.insert("version", "1.0".to_string());
                params.insert("attr_names", "name,age".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            _ensure_schema_added(&ctx);
            close_and_delete_wallet(&ctx);
            disconnect_and_delete_pool(&ctx);
        }
    }

    mod get_schema {
        use super::*;

        #[test]
        pub fn schema_works() {
            let ctx = CommandContext::new();

            create_and_open_wallet(&ctx);
            create_and_connect_pool(&ctx);

            new_did(&ctx, SEED_TRUSTEE);
            use_did(&ctx, DID_TRUSTEE);
            {
                let cmd = schema_command::new();
                let mut params = CommandParams::new();
                params.insert("name", "gvt".to_string());
                params.insert("version", "1.0".to_string());
                params.insert("attr_names", "name,age".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            {
                let cmd = get_schema_command::new();
                let mut params = CommandParams::new();
                params.insert("did", DID_TRUSTEE.to_string());
                params.insert("name", "gvt".to_string());
                params.insert("version", "1.0".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            _ensure_schema_added(&ctx);
            close_and_delete_wallet(&ctx);
            disconnect_and_delete_pool(&ctx);
        }
    }

    mod claim_def {
        use super::*;

        #[test]
        pub fn claim_def_works() {
            let ctx = CommandContext::new();

            create_and_open_wallet(&ctx);
            create_and_connect_pool(&ctx);

            new_did(&ctx, SEED_TRUSTEE);
            use_did(&ctx, DID_TRUSTEE);
            {
                let cmd = claim_def_command::new();
                let mut params = CommandParams::new();
                params.insert("schema_no", "1".to_string());
                params.insert("signature_type", "CL".to_string());
                params.insert("primary", r#"{"n":"1","s":"1","rms":"1","r":{"age":"1","name":"1"},"rctxt":"1","z":"1"}"#.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            _ensure_claim_def_added(&ctx);
            close_and_delete_wallet(&ctx);
            disconnect_and_delete_pool(&ctx);
        }
    }

    mod get_claim_def {
        use super::*;

        #[test]
        pub fn get_claim_def_works() {
            let ctx = CommandContext::new();

            create_and_open_wallet(&ctx);
            create_and_connect_pool(&ctx);

            new_did(&ctx, SEED_TRUSTEE);
            use_did(&ctx, DID_TRUSTEE);
            {
                let cmd = claim_def_command::new();
                let mut params = CommandParams::new();
                params.insert("schema_no", "1".to_string());
                params.insert("signature_type", "CL".to_string());
                params.insert("primary", r#"{"n":"1","s":"1","rms":"1","r":{"age":"1","name":"1"},"rctxt":"1","z":"1"}"#.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            {
                let cmd = get_claim_def_command::new();
                let mut params = CommandParams::new();
                params.insert("schema_no", "1".to_string());
                params.insert("signature_type", "CL".to_string());
                params.insert("origin", DID_TRUSTEE.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            close_and_delete_wallet(&ctx);
            disconnect_and_delete_pool(&ctx);
        }
    }

    mod node {
        use super::*;

        #[test]
        pub fn node_works() {
            let ctx = CommandContext::new();

            let my_seed = "00000000000000000000000MySTEWARD";
            let my_did = "GykzQ65PxaH3RUDypuwWTB";
            let my_verkey = "9i7fMkxTSdTaHkTmLqZ3exRkTfsQ5LLoxzDG1kjE8HLD";

            create_and_open_wallet(&ctx);
            create_and_connect_pool(&ctx);

            new_did(&ctx, SEED_TRUSTEE);
            new_did(&ctx, my_seed);
            use_did(&ctx, DID_TRUSTEE);
            send_nym(&ctx, my_did, my_verkey, Some("STEWARD"));
            use_did(&ctx, my_did);
            {
                let cmd = node_command::new();
                let mut params = CommandParams::new();
                params.insert("target", "A5iWQVT3k8Zo9nXj4otmeqaUziPQPCiDqcydXkAJBk1Y".to_string());
                params.insert("node_ip", "127.0.0.1".to_string());
                params.insert("node_port", "9710".to_string());
                params.insert("client_ip", "127.0.0.2".to_string());
                params.insert("client_port", "9711".to_string());
                params.insert("alias", "Node5".to_string());
                params.insert("blskey", "2zN3bHM1m4rLz54MJHYSwvqzPchYp8jkHswveCLAEJVcX6Mm1wHQD1SkPYMzUDTZvWvhuE6VNAkK3KxVeEmsanSmvjVkReDeBEMxeDaayjcZjFGPydyey1qxBHmTvAnBKoPydvuTAqx5f7YNNRAdeLmUi99gERUU7TD8KfAa6MpQ9bw".to_string());
                params.insert("services", "VALIDATOR".to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            close_and_delete_wallet(&ctx);
            disconnect_and_delete_pool(&ctx);
        }
    }

    mod custom {
        use super::*;

        #[test]
        pub fn custom_works() {
            let ctx = CommandContext::new();

            create_and_open_wallet(&ctx);
            create_and_connect_pool(&ctx);

            new_did(&ctx, SEED_TRUSTEE);
            use_did(&ctx, DID_TRUSTEE);
            {
                let cmd = custom_command::new();
                let mut params = CommandParams::new();
                params.insert("txn", format!(r#"{{
                                                    "reqId":1513241300414292814,
                                                    "identifier":"{}",
                                                    "operation":{{
                                                        "type":"105",
                                                        "dest":"{}"
                                                    }},
                                                    "protocolVersion":1
                                                  }}"#, DID_TRUSTEE, DID_TRUSTEE));
                cmd.execute(&ctx, &params).unwrap();
            }
            close_and_delete_wallet(&ctx);
            disconnect_and_delete_pool(&ctx);
        }

        #[test]
        pub fn custom_works_for_sign() {
            let ctx = CommandContext::new();

            create_and_open_wallet(&ctx);
            create_and_connect_pool(&ctx);

            new_did(&ctx, SEED_TRUSTEE);
            use_did(&ctx, DID_TRUSTEE);
            {
                let cmd = custom_command::new();
                let mut params = CommandParams::new();
                params.insert("sign", "true".to_string());
                params.insert("txn", format!(r#"{{
                                                    "reqId":1513241300414292814,
                                                    "identifier":"{}",
                                                    "operation":{{
                                                        "type":"1",
                                                        "dest":"{}",
                                                        "verkey":"{}"
                                                    }},
                                                    "protocolVersion":1
                                                  }}"#, DID_TRUSTEE, DID_MY1, VERKEY_MY1));
                cmd.execute(&ctx, &params).unwrap();
            }
            close_and_delete_wallet(&ctx);
            disconnect_and_delete_pool(&ctx);
        }
    }

    use std::sync::{Once, ONCE_INIT};

    pub fn send_nym_my1(ctx: &CommandContext) {
        lazy_static! {
            static ref SEND_NYM: Once = ONCE_INIT;

        }

        SEND_NYM.call_once(|| {
            let cmd = nym_command::new();
            let mut params = CommandParams::new();
            params.insert("did", DID_MY1.to_string());
            params.insert("verkey", VERKEY_MY1.to_string());
            cmd.execute(&ctx, &params).unwrap();
        });
    }

    pub fn send_nym(ctx: &CommandContext, did: &str, verkey: &str, role: Option<&str>) {
        let cmd = nym_command::new();
        let mut params = CommandParams::new();
        params.insert("did", did.to_string());
        params.insert("verkey", verkey.to_string());
        if let Some(role) = role {
            params.insert("role", role.to_string());
        }
        cmd.execute(&ctx, &params).unwrap();
    }

    fn _ensure_nym_added(ctx: &CommandContext) {
        let request = Ledger::build_get_nym_request(DID_TRUSTEE, DID_MY1).unwrap();
        let pool_handle = ensure_connected_pool_handle(&ctx).unwrap();
        let response = Ledger::submit_request(pool_handle, &request).unwrap();
        serde_json::from_str::<Reply<String>>(&response)
            .and_then(|response| serde_json::from_str::<NymData>(&response.result.data)).unwrap();
    }

    fn _ensure_attrib_added(ctx: &CommandContext) {
        let request = Ledger::build_get_attrib_request(DID_MY1, DID_MY1, "endpoint").unwrap();
        let pool_handle = ensure_connected_pool_handle(&ctx).unwrap();
        let response = Ledger::submit_request(pool_handle, &request).unwrap();
        serde_json::from_str::<Reply<String>>(&response)
            .and_then(|response| serde_json::from_str::<AttribData>(&response.result.data)).unwrap();
    }

    fn _ensure_schema_added(ctx: &CommandContext) {
        let data = r#"{"name":"gvt", "version":"1.0"}"#;
        let request = Ledger::build_get_schema_request(DID_TRUSTEE, DID_TRUSTEE, data).unwrap();
        let pool_handle = ensure_connected_pool_handle(&ctx).unwrap();
        let response = Ledger::submit_request(pool_handle, &request).unwrap();
        serde_json::from_str::<Reply<SchemaData>>(&response).unwrap();
    }

    fn _ensure_claim_def_added(ctx: &CommandContext) {
        let request = Ledger::build_get_claim_def_txn(DID_TRUSTEE, 1, "CL", DID_TRUSTEE).unwrap();
        let pool_handle = ensure_connected_pool_handle(&ctx).unwrap();
        let response = Ledger::submit_request(pool_handle, &request).unwrap();
        serde_json::from_str::<Reply<ClaimDefData>>(&response).unwrap();
    }
}