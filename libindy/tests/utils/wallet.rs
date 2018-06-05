use indy::api::ErrorCode;
use indy::api::wallet::*;

use utils::callback::CallbackUtils;
use utils::inmem_wallet::InmemWallet;
use utils::sequence::SequenceUtils;

use std::collections::HashSet;
use std::ffi::CString;
use std::ptr::null;
use std::sync::Mutex;
use utils::constants::DEFAULT_WALLET_CREDENTIALS;

pub struct WalletUtils {}

impl WalletUtils {
    pub fn register_wallet_type(xtype: &str, force_create: bool) -> Result<(), ErrorCode> {
        unimplemented!()
        //        lazy_static! {
        //            static ref REGISERED_WALLETS: Mutex<HashSet<String>> = Default::default();
        //        }
        //
        //        let mut wallets = REGISERED_WALLETS.lock().unwrap();
        //
        //        if wallets.contains(xtype) & !force_create {
        //            // as registering of plugged wallet with
        //            return Ok(());
        //        }
        //
        //        let (receiver, command_handle, cb) = CallbackUtils::_closure_to_cb_ec();
        //
        //        let xxtype = CString::new(xtype).unwrap();
        //
        //        let err = indy_register_wallet_storage(
        //            command_handle,
        //            xxtype.as_ptr(),
        //            Some(InmemWallet::create),
        //            Some(InmemWallet::open),
        //            Some(InmemWallet::set),
        //            Some(InmemWallet::get),
        //            Some(InmemWallet::get_not_expired),
        //            Some(InmemWallet::list),
        //            Some(InmemWallet::close),
        //            Some(InmemWallet::delete),
        //            Some(InmemWallet::free),
        //            cb
        //        );
        //
        //        wallets.insert(xtype.to_string());
        //
        //        super::results::result_to_empty(err, receiver)
    }

    pub fn create_wallet(pool_name: &str, wallet_name: &str, xtype: Option<&str>, config: Option<&str>, credentials: Option<&str>) -> Result<(), ErrorCode> {
        let (receiver, command_handle, cb) = CallbackUtils::_closure_to_cb_ec();

        let pool_name = CString::new(pool_name).unwrap();
        let wallet_name = CString::new(wallet_name).unwrap();
        let xtype_str = xtype.map(|s| CString::new(s).unwrap()).unwrap_or(CString::new("").unwrap());
        let config_str = config.map(|s| CString::new(s).unwrap()).unwrap_or(CString::new("").unwrap());
        let credentials_str = CString::new(credentials.unwrap_or(DEFAULT_WALLET_CREDENTIALS)).unwrap();

        let err =
            indy_create_wallet(command_handle,
                               pool_name.as_ptr(),
                               wallet_name.as_ptr(),
                               if xtype.is_some() { xtype_str.as_ptr() } else { null() },
                               if config.is_some() { config_str.as_ptr() } else { null() },
                               credentials_str.as_ptr(),
                               cb);

        super::results::result_to_empty(err, receiver)
    }

    pub fn open_wallet(wallet_name: &str, config: Option<&str>, credentials: Option<&str>) -> Result<i32, ErrorCode> {
        let (receiver, command_handle, cb) = CallbackUtils::_closure_to_cb_ec_i32();

        let wallet_name = CString::new(wallet_name).unwrap();
        let config_str = config.map(|s| CString::new(s).unwrap()).unwrap_or(CString::new("").unwrap());
        let credentials_str = CString::new(credentials.unwrap_or(DEFAULT_WALLET_CREDENTIALS)).unwrap();

        let err =
            indy_open_wallet(command_handle,
                             wallet_name.as_ptr(),
                             if config.is_some() { config_str.as_ptr() } else { null() },
                             credentials_str.as_ptr(),
                             cb);

        super::results::result_to_int(err, receiver)
    }

    pub fn create_and_open_wallet(pool_name: &str, xtype: Option<&str>) -> Result<i32, ErrorCode> {
        let wallet_name = format!("default-wallet-name-{}", SequenceUtils::get_next_id());

        WalletUtils::create_wallet(pool_name, &wallet_name, xtype, None, None)?;
        WalletUtils::open_wallet(&wallet_name, None, None)
    }

    pub fn delete_wallet(wallet_name: &str) -> Result<(), ErrorCode> {
        let (receiver, command_handle, cb) = CallbackUtils::_closure_to_cb_ec();

        let wallet_name = CString::new(wallet_name).unwrap();
        let credentials_str = CString::new(DEFAULT_WALLET_CREDENTIALS).unwrap();

        let err = indy_delete_wallet(command_handle, wallet_name.as_ptr(), credentials_str.as_ptr(), cb);

        super::results::result_to_empty(err, receiver)
    }

    pub fn close_wallet(wallet_handle: i32) -> Result<(), ErrorCode> {
        let (receiver, command_handle, cb) = CallbackUtils::_closure_to_cb_ec();

        let err = indy_close_wallet(command_handle, wallet_handle, cb);

        super::results::result_to_empty(err, receiver)
    }

    pub fn export_wallet(wallet_handle: i32, export_path: &str, key: &str) -> Result<(), ErrorCode> {
        let (receiver, command_handle, cb) = CallbackUtils::_closure_to_cb_ec();
        let export_path = CString::new(export_path).unwrap();
        let key = CString::new(key).unwrap();

        let err = indy_export_wallet(command_handle, wallet_handle, export_path.as_ptr(), key.as_ptr(),cb);

        super::results::result_to_empty(err, receiver)
    }
}