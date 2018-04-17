extern crate libc;

use api::ErrorCode;
use errors::ToErrorCode;
use commands::{Command, CommandExecutor};
use commands::ledger::LedgerCommand;
use utils::cstring::CStringUtils;

use self::libc::c_char;

/// Signs and submits request message to validator pool.
///
/// Adds submitter information to passed request json, signs it with submitter
/// sign key (see wallet_sign), and sends signed request message
/// to validator pool (see write_request).
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// pool_handle: pool handle (created by open_pool_ledger).
/// wallet_handle: wallet handle (created by open_wallet).
/// submitter_did: Id of Identity stored in secured Wallet.
/// request_json: Request data json.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
/// Wallet*
/// Ledger*
/// Crypto*
#[no_mangle]
pub extern fn indy_sign_and_submit_request(command_handle: i32,
                                           pool_handle: i32,
                                           wallet_handle: i32,
                                           submitter_did: *const c_char,
                                           request_json: *const c_char,
                                           cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                                request_result_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(request_json, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::SignAndSubmitRequest(
            pool_handle,
            wallet_handle,
            submitter_did,
            request_json,
            Box::new(move |result| {
                let (err, request_result_json) = result_to_err_code_1!(result, String::new());
                let request_result_json = CStringUtils::string_to_cstring(request_result_json);
                cb(command_handle, err, request_result_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Publishes request message to validator pool (no signing, unlike sign_and_submit_request).
///
/// The request is sent to the validator pool as is. It's assumed that it's already prepared.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// pool_handle: pool handle (created by open_pool_ledger).
/// request_json: Request data json.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
/// Ledger*
#[no_mangle]
pub extern fn indy_submit_request(command_handle: i32,
                                  pool_handle: i32,
                                  request_json: *const c_char,
                                  cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                       request_result_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(request_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::SubmitRequest(
            pool_handle,
            request_json,
            Box::new(move |result| {
                let (err, request_result_json) = result_to_err_code_1!(result, String::new());
                let request_result_json = CStringUtils::string_to_cstring(request_result_json);
                cb(command_handle, err, request_result_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Signs request message.
///
/// Adds submitter information to passed request json, signs it with submitter
/// sign key (see wallet_sign).
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// wallet_handle: wallet handle (created by open_wallet).
/// submitter_did: Id of Identity stored in secured Wallet.
/// request_json: Request data json.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Signed request json.
///
/// #Errors
/// Common*
/// Wallet*
/// Ledger*
/// Crypto*
#[no_mangle]
pub extern fn indy_sign_request(command_handle: i32,
                                wallet_handle: i32,
                                submitter_did: *const c_char,
                                request_json: *const c_char,
                                cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                     signed_request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(request_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::SignRequest(
            wallet_handle,
            submitter_did,
            request_json,
            Box::new(move |result| {
                let (err, signed_request_json) = result_to_err_code_1!(result, String::new());
                let signed_request_json = CStringUtils::string_to_cstring(signed_request_json);
                cb(command_handle, err, signed_request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}


/// Builds a request to get a DDO.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// target_did: Id of Identity stored in secured Wallet.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_ddo_request(command_handle: i32,
                                         submitter_did: *const c_char,
                                         target_did: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                              request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(target_did, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetDdoRequest(
            submitter_did,
            target_did,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}


/// Builds a NYM request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// target_did: Id of Identity stored in secured Wallet.
/// verkey: verification key
/// alias: alias
/// role: Role of a user NYM record
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_nym_request(command_handle: i32,
                                     submitter_did: *const c_char,
                                     target_did: *const c_char,
                                     verkey: *const c_char,
                                     alias: *const c_char,
                                     role: *const c_char,
                                     cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                          request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(target_did, ErrorCode::CommonInvalidParam3);
    check_useful_opt_c_str!(verkey, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(alias, ErrorCode::CommonInvalidParam5);
    check_useful_opt_c_str!(role, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildNymRequest(
            submitter_did,
            target_did,
            verkey,
            alias,
            role,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Builds an ATTRIB request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// target_did: Id of Identity stored in secured Wallet.
/// hash: Hash of attribute data
/// raw: represented as json, where key is attribute name and value is it's value
/// enc: Encrypted attribute data
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_attrib_request(command_handle: i32,
                                        submitter_did: *const c_char,
                                        target_did: *const c_char,
                                        hash: *const c_char,
                                        raw: *const c_char,
                                        enc: *const c_char,
                                        cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                             request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(target_did, ErrorCode::CommonInvalidParam3);
    check_useful_opt_c_str!(hash, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(raw, ErrorCode::CommonInvalidParam5);
    check_useful_opt_c_str!(enc, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildAttribRequest(
            submitter_did,
            target_did,
            hash,
            raw,
            enc,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Builds a GET_ATTRIB request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// target_did: Id of Identity stored in secured Wallet.
/// data: name (attribute name)
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_attrib_request(command_handle: i32,
                                            submitter_did: *const c_char,
                                            target_did: *const c_char,
                                            raw: *const c_char,
                                            hash: *const c_char,
                                            enc: *const c_char,
                                            cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                                 request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(target_did, ErrorCode::CommonInvalidParam3);
    check_useful_opt_c_str!(raw, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(hash, ErrorCode::CommonInvalidParam5);
    check_useful_opt_c_str!(enc, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetAttribRequest(
            submitter_did,
            target_did,
            raw,
            hash,
            enc,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Builds a GET_NYM request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// target_did: Id of Identity stored in secured Wallet.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_nym_request(command_handle: i32,
                                         submitter_did: *const c_char,
                                         target_did: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                              request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(target_did, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetNymRequest(
            submitter_did,
            target_did,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Builds a SCHEMA request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// data: name, version, type, attr_names (ip, port, keys)
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_schema_request(command_handle: i32,
                                        submitter_did: *const c_char,
                                        data: *const c_char,
                                        cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                             request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(data, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildSchemaRequest(
            submitter_did,
            data,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Builds a GET_SCHEMA request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// dest: Id of Identity stored in secured Wallet.
/// data: name, version
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_schema_request(command_handle: i32,
                                            submitter_did: *const c_char,
                                            dest: *const c_char,
                                            data: *const c_char,
                                            cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                                 request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(dest, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(data, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetSchemaRequest(
            submitter_did,
            dest,
            data,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Builds an CLAIM_DEF request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// xref: Seq. number of schema
/// signature_type: signature type (only CL supported now)
/// data: components of a key in json: N, R, S, Z
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_claim_def_txn(command_handle: i32,
                                       submitter_did: *const c_char,
                                       xref: i32,
                                       signature_type: *const c_char,
                                       data: *const c_char,
                                       cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                            request_result_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(signature_type, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(data, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildClaimDefRequest(
            submitter_did,
            xref,
            signature_type,
            data,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Builds a GET_CLAIM_DEF request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// xref: Seq. number of schema
/// signature_type: signature type (only CL supported now)
/// origin: issuer did
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_claim_def_txn(command_handle: i32,
                                           submitter_did: *const c_char,
                                           xref: i32,
                                           signature_type: *const c_char,
                                           origin: *const c_char,
                                           cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                                request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(signature_type, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(origin, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetClaimDefRequest(
            submitter_did,
            xref,
            signature_type,
            origin,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Builds a NODE request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// target_did: Id of Identity stored in secured Wallet.
/// data: id of a target NYM record
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_node_request(command_handle: i32,
                                      submitter_did: *const c_char,
                                      target_did: *const c_char,
                                      data: *const c_char,
                                      cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                           request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(target_did, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(data, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildNodeRequest(
            submitter_did,
            target_did,
            data,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Builds a GET_VALIDATOR_INFO request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_validator_info_request(command_handle: i32,
                                         submitter_did: *const c_char,
                                         cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                              request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetValidatorInfoRequest(
            submitter_did,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Builds a GET_TXN request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// data: seq_no of transaction in ledger
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_txn_request(command_handle: i32,
                                         submitter_did: *const c_char,
                                         data: i32,
                                         cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                              request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetTxnRequest(
            submitter_did,
            data,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Builds a POOL_CONFIG request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// writes:
/// force:
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_pool_config_request(command_handle: i32,
                                             submitter_did: *const c_char,
                                             writes: bool,
                                             force: bool,
                                             cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                                  request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildPoolConfigRequest(
            submitter_did,
            writes,
            force,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}

/// Builds a POOL_RESTART request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// action:
/// datetime:
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_pool_restart_request(command_handle: i32,
                                              submitter_did: *const c_char,
                                              action: *const c_char,
                                              datetime: *const c_char,
                                              cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                                  request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(action, ErrorCode::CommonInvalidParam3);
    check_useful_opt_c_str!(datetime, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildPoolRestartRequest(submitter_did,
                                                                     action,
                                                                     datetime,
                                                                     Box::new(move |result| {
                                                                         let (err, request_json) = result_to_err_code_1!(result, String::new());
                                                                         let request_json = CStringUtils::string_to_cstring(request_json);
                                                                         cb(command_handle, err, request_json.as_ptr())
                                                                     }))));

    result_to_err_code!(result)
}


/// Builds a POOL_UPGRADE request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Id of Identity stored in secured Wallet.
/// name:
/// action: Either start or cancel
/// sha256:
/// timeout:
/// schedule:
/// justification:
/// reinstall:
/// force:
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_pool_upgrade_request(command_handle: i32,
                                              submitter_did: *const c_char,
                                              name: *const c_char,
                                              version: *const c_char,
                                              action: *const c_char,
                                              sha256: *const c_char,
                                              timeout: i32,
                                              schedule: *const c_char,
                                              justification: *const c_char,
                                              reinstall: bool,
                                              force: bool,
                                              cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                                   request_json: *const c_char)>) -> ErrorCode {
    check_useful_c_str!(submitter_did, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(name, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(version, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(action, ErrorCode::CommonInvalidParam5);
    check_useful_c_str!(sha256, ErrorCode::CommonInvalidParam6);
    check_useful_opt_c_str!(schedule, ErrorCode::CommonInvalidParam8);
    check_useful_opt_c_str!(justification, ErrorCode::CommonInvalidParam9);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam12);

    let timeout = if timeout != -1 { Some(timeout as u32) } else { None };


    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildPoolUpgradeRequest(
            submitter_did,
            name,
            version,
            action,
            sha256,
            timeout,
            schedule,
            justification,
            reinstall,
            force,
            Box::new(move |result| {
                let (err, request_json) = result_to_err_code_1!(result, String::new());
                let request_json = CStringUtils::string_to_cstring(request_json);
                cb(command_handle, err, request_json.as_ptr())
            })
        )));

    result_to_err_code!(result)
}
