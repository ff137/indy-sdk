extern crate libc;

use self::libc::c_char;
use utils::cstring::CStringUtils;
use std::ptr::null;
use utils::generate_command_handle;
use utils::init::indy_error_to_cxs_error_code;

extern {
    fn indy_create_wallet(command_handle: i32,
                                 pool_name: *const c_char,
                                 name: *const c_char,
                                 xtype: *const c_char,
                                 config: *const c_char,
                                 credentials: *const c_char,
                                 cb: Option<extern fn(xcommand_handle: i32, err: i32)>) -> i32;
}


pub fn create_wallet<'a>(pool_name:&str, wallet_name:&str, wallet_type:&str) -> u32 {
    let handle = generate_command_handle();

    // currently we have no call backs
    extern "C" fn dummy_callback(_handle: i32, _err: i32) { }

    unsafe {
        let indy_err = indy_create_wallet(handle,
                                          CStringUtils::string_to_cstring(pool_name.to_string()).as_ptr(),
                                          CStringUtils::string_to_cstring(wallet_name.to_string()).as_ptr(),
                                          CStringUtils::string_to_cstring(wallet_type.to_string()).as_ptr(),
                                          null(),
                                          null(),
                                          Some(dummy_callback));
        indy_error_to_cxs_error_code(indy_err)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use utils::error;
    #[test]
    fn test_wallet() {
        let pool_name = String::from("pool1");
        let wallet_name = String::from("wallet1");
        let wallet_type = String::from("default");
        assert_eq!(error::SUCCESS.code_num, create_wallet(&pool_name, &wallet_name, &wallet_type));
        assert_eq!(error::UNKNOWN_ERROR.code_num, create_wallet(&String::from(""),&wallet_name, &wallet_type));

    }
}
