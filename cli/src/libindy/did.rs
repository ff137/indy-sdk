use super::{ErrorCode, IndyHandle};

use libc::c_char;
use std::ffi::CString;
use std::sync::mpsc::channel;


pub struct Did {}

impl Did {
    pub fn new(wallet_handle: IndyHandle, my_did_json: &str) -> Result<(String, String), ErrorCode> {
        let (sender, receiver) = channel();

        let (command_handle, cb) =  super::callbacks::_closure_to_cb_ec_string_string(sender);

        let my_did_json = CString::new(my_did_json).unwrap();

        let err = unsafe {
            indy_create_and_store_my_did(command_handle,
                                         wallet_handle,
                                         my_did_json.as_ptr(),
                                         cb)
        };

        super::results::result_to_string_string(err, receiver)
    }

    pub fn replace_keys_start(wallet_handle: i32, did: &str, identity_json: &str) -> Result<String, ErrorCode> {
        let (sender, receiver) = channel();

        let (command_handle, cb) = super::callbacks::_closure_to_cb_ec_string(sender);

        let did = CString::new(did).unwrap();
        let identity_json = CString::new(identity_json).unwrap();

        let err = unsafe {
            indy_replace_keys_start(command_handle,
                                    wallet_handle,
                                    did.as_ptr(),
                                    identity_json.as_ptr(),
                                    cb)
        };

        super::results::result_to_string(err, receiver)
    }

    pub fn replace_keys_apply(wallet_handle: i32, did: &str) -> Result<(), ErrorCode> {
        let (sender, receiver) = channel();

        let (command_handle, cb) = super::callbacks::_closure_to_cb_ec(sender);

        let did = CString::new(did).unwrap();

        let err = unsafe {
            indy_replace_keys_apply(command_handle,
                                    wallet_handle,
                                    did.as_ptr(),
                                    cb)
        };

        super::results::result_to_empty(err, receiver)
    }

    pub fn set_metadata(wallet_handle: i32, did: &str, metadata: &str) -> Result<(), ErrorCode> {
        let (sender, receiver) = channel();

        let (command_handle, callback) = super::callbacks::_closure_to_cb_ec(sender);

        let did = CString::new(did).unwrap();
        let metadata = CString::new(metadata).unwrap();

        let err = unsafe {
            indy_set_did_metadata(command_handle,
                                  wallet_handle,
                                  did.as_ptr(),
                                  metadata.as_ptr(),
                                  callback)
        };

        super::results::result_to_empty(err, receiver)
    }

    pub fn get_did_with_meta(wallet_handle: i32, did: &str) -> Result<String, ErrorCode> {
        let (sender, receiver) = channel();

        let (command_handle, cb) = super::callbacks::_closure_to_cb_ec_string(sender);

        let did = CString::new(did).unwrap();

        let err = unsafe {
            indy_get_my_did_with_meta(command_handle,
                                      wallet_handle,
                                      did.as_ptr(),
                                      cb)
        };

        super::results::result_to_string(err, receiver)
    }

    pub fn list_dids_with_meta(wallet_handle: i32) -> Result<String, ErrorCode> {
        let (sender, receiver) = channel();

        let (command_handle, cb) = super::callbacks::_closure_to_cb_ec_string(sender);

        let err = unsafe { indy_list_my_dids_with_meta(command_handle, wallet_handle, cb) };

        super::results::result_to_string(err, receiver)
    }
}

extern {
    #[no_mangle]
    pub fn indy_create_and_store_my_did(command_handle: i32,
                                        wallet_handle: i32,
                                        did_json: *const c_char,
                                        cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                             did: *const c_char,
                                                             verkey: *const c_char)>) -> ErrorCode;

    #[no_mangle]
    pub fn indy_replace_keys_start(command_handle: i32,
                                   wallet_handle: i32,
                                   did: *const c_char,
                                   identity_json: *const c_char,
                                   cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                        verkey: *const c_char)>) -> ErrorCode;

    #[no_mangle]
    pub fn indy_replace_keys_apply(command_handle: i32,
                                   wallet_handle: i32,
                                   did: *const c_char,
                                   cb: Option<extern fn(xcommand_handle: i32,
                                                        err: ErrorCode)>) -> ErrorCode;

    #[no_mangle]
    fn indy_set_did_metadata(command_handle: i32,
                             wallet_handle: i32,
                             did: *const c_char,
                             metadata: *const c_char,
                             cb: Option<extern fn(command_handle_: i32,
                                                  err: ErrorCode)>) -> ErrorCode;

    #[no_mangle]
    pub fn indy_get_my_did_with_meta(command_handle: i32,
                                     wallet_handle: i32,
                                     my_did: *const c_char,
                                     cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                          did_with_meta: *const c_char)>) -> ErrorCode;

    #[no_mangle]
    fn indy_list_my_dids_with_meta(command_handle: i32,
                                   wallet_handle: i32,
                                   cb: Option<extern fn(xcommand_handle: i32, err: ErrorCode,
                                                        dids: *const c_char)>) -> ErrorCode;
}