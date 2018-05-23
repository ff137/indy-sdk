use super::{ErrorCode, IndyHandle};

use std::ffi::CString;

use ffi::crypto;

use utils::results::ResultHandler;
use utils::callbacks::ClosureHandler;

pub struct Key {}

impl Key {
    /// Creates key pair in wallet
    /// # Arguments
    /// * `wallet_handle` - wallet handle (created by Wallet::open)
    /// * `my_key_json` - key information as json
    ///
    /// # Example
    /// my_key_json
    /// {
    ///     "seed": string, // Optional (if not set random one will be used); Seed information that allows deterministic key creation.
    ///     "crypto_type": string, // Optional (if not set then ed25519 curve is used); Currently only 'ed25519' value is supported for this field.
    /// }
    /// # Returns
    /// verkey of generated key pair, also used as key identifier
    pub fn create(wallet_handle: IndyHandle, my_key_json: &str) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) = ClosureHandler::cb_ec_string();

        let my_key_json = c_str!(my_key_json);

        let err = unsafe {
            crypto::indy_create_key(command_handle, wallet_handle, my_key_json.as_ptr(), cb)
        };
        ResultHandler::one(err, receiver)
    }

    /// Saves/replaces the metadata for the `verkey` in the wallet
    /// # Arguments
    /// * `wallet_handle` - wallet handle (created by Wallet::open)
    /// * `verkey` - the public key or key id where to store the metadata
    /// * `metadata` - the metadata that will be stored with the key
    pub fn set_metadata(wallet_handle: IndyHandle, verkey: &str, metadata: &str) -> Result<(), ErrorCode> {
        let (receiver, command_handle, cb) = ClosureHandler::cb_ec();

        let verkey = c_str!(verkey);
        let metadata = c_str!(metadata);

        let err = unsafe {
            crypto::indy_set_key_metadata(command_handle, wallet_handle, verkey.as_ptr(), metadata.as_ptr(), cb)
        };
        ResultHandler::empty(err, receiver)
    }

    /// Retrieves the metadata for the `verkey` in the wallet
    /// # Argument
    /// * `wallet_handle` - wallet handle (created by Wallet::open)
    /// * `verkey` - the public key or key id to retrieve metadata
    /// # Returns
    /// metadata currently stored with the key; Can be empty if no metadata was saved for this key
    pub fn get_metadata(wallet_handle: IndyHandle, verkey: &str) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) = ClosureHandler::cb_ec_string();

        let verkey = c_str!(verkey);

        let err = unsafe {
            crypto::indy_get_key_metadata(command_handle, wallet_handle, verkey.as_ptr(), cb)
        };

        ResultHandler::one(err, receiver)
    }
}

pub struct Crypto {}

impl Crypto {
    /// Signs a message with a key
    /// # Arguments
    /// * `wallet_handle` - wallet handle (created by Wallet::open)
    /// * `signer_vk` - key id or verkey of my key. The key must be created by calling Key::create or Did::new
    /// * `message` - the data to be signed
    /// # Returns
    /// the signature
    pub fn sign(wallet_handle: IndyHandle, signer_vk: &str, message: &[u8]) -> Result<Vec<u8>, ErrorCode> {
        let (receiver, command_handle, cb) = ClosureHandler::cb_ec_slice();

        let signer_vk = c_str!(signer_vk);
        let err = unsafe {
            crypto::indy_crypto_sign(command_handle, wallet_handle, signer_vk.as_ptr(),
                             message.as_ptr() as *const u8,
                             message.len() as u32,
                             cb)
        };

        ResultHandler::one(err, receiver)
    }

    /// Verify a signature with a verkey
    /// # Arguments
    /// * `wallet_handle` - wallet handle (created by Wallet::open)
    /// * `signer_vk` - key id or verkey of my key. The key must be created by calling Key::create or Did::new
    /// * `message` - the data that was signed
    /// * `signature` - the signature to verify
    /// # Returns
    /// true if signature is valid, false otherwise
    pub fn verify(wallet_handle: IndyHandle, signer_vk: &str, message: &[u8], signature: &[u8]) -> Result<bool, ErrorCode> {
        let (receiver, command_handle, cb) = ClosureHandler::cb_ec_bool();

        let signer_vk = c_str!(signer_vk);

        let err = unsafe {
            crypto::indy_crypto_verify(command_handle, wallet_handle, signer_vk.as_ptr(),
                               message.as_ptr() as *const u8, message.len() as u32,
                               signature.as_ptr() as *const u8, signature.len() as u32, cb)
        };

        ResultHandler::one(err, receiver)
    }

    /// Encrypt a message by authenticated-encryption scheme.
    ///
    /// Sender can encrypt a confidential message specifically for Recipient, using Sender's public key.
    /// Using Recipient's public key, Sender can compute a shared secret key.
    /// Using Sender's public key and his secret key, Recipient can compute the exact same shared secret key.
    /// That shared secret key can be used to verify that the encrypted message was not tampered with,
    /// before eventually decrypting it.
    ///
    /// Note to use DID keys with this function you can call Did::get_ver_key to get key id (verkey)
    /// for specific DID.
    /// # Arguments
    /// * `wallet_handle` - wallet handle (created by Wallet::open)
    /// * `signer_vk` - key id or verkey of my key. The key must be created by calling Key::create or Did::new
    /// * `recipient_vk` - key id or verkey of the other party's key
    /// * `message` - the data to be encrypted
    /// # Returns
    /// the encrypted message
    pub fn auth_crypt(wallet_handle: IndyHandle, sender_vk: &str, recipient_vk: &str, message: &[u8]) -> Result<Vec<u8>, ErrorCode> {
        let (receiver, command_handle, cb) = ClosureHandler::cb_ec_slice();

        let sender_vk = c_str!(sender_vk);
        let recipient_vk = c_str!(recipient_vk);
        let err = unsafe {
            crypto::indy_crypto_auth_crypt(command_handle, wallet_handle,
                                   sender_vk.as_ptr(),
                                    recipient_vk.as_ptr(),
                                    message.as_ptr() as *const u8,
                                    message.len() as u32, cb)
        };
        ResultHandler::one(err, receiver)
    }

    pub fn auth_decrypt(wallet_handle: IndyHandle, recipient_vk: &str, encrypted_message: &[u8]) -> Result<(String, Vec<u8>), ErrorCode> {
        let (receiver, command_handle, cb) = ClosureHandler::cb_ec_string_slice();

        let recipient_vk = c_str!(recipient_vk);
        let err = unsafe {
            crypto::indy_crypto_auth_decrypt(command_handle,
                                     wallet_handle,
                                     recipient_vk.as_ptr(),
                                     encrypted_message.as_ptr() as *const u8,
                                     encrypted_message.len() as u32, cb)
        };

        ResultHandler::two(err, receiver)
    }

    pub fn anon_crypt(wallet_handle: IndyHandle, recipient_vk: &str, message: &[u8]) -> Result<Vec<u8>, ErrorCode> {
        let (receiver, command_handle, cb) = ClosureHandler::cb_ec_slice();

        let recipient_vk = c_str!(recipient_vk);
        let err = unsafe {
            crypto::indy_crypto_anon_crypt(command_handle,
                                   wallet_handle,
                                   recipient_vk.as_ptr(),
                                   message.as_ptr() as *const u8,
                                    message.len() as u32,
                                    cb)
        };

        ResultHandler::one(err, receiver)
    }

    pub fn anon_decrypt(wallet_handle: IndyHandle, recipient_vk: &str, encrypted_message: &[u8]) -> Result<(String, Vec<u8>), ErrorCode> {
        let (receiver, command_handle, cb) = ClosureHandler::cb_ec_string_slice();

        let recipient_vk = c_str!(recipient_vk);
        let err = unsafe {
            crypto::indy_crypto_anon_decrypt(command_handle,
                                     wallet_handle,
                                     recipient_vk.as_ptr(),
                                     encrypted_message.as_ptr() as *const u8,
                                     encrypted_message.len() as u32, cb)
        };

        ResultHandler::two(err, receiver)
    }
}
