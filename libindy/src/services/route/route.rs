use services::route::jwm;
use services::route::jwm::{JWM, Header, Recipient};
use services::route::jwm_crypto;
use services::route::route_support;
use errors::route::RouteError;
use errors::common::CommonError;
use services::crypto::CryptoService;
use services::route::route_table::RouteTable;
use utils::crypto::base64::{decode, encode};
use domain::crypto::combo_box::ComboBox;
use domain::crypto::key::Key;
use services::wallet::{WalletService, RecordOptions};


pub struct RouteService {
    wallet_service: WalletService,
    crypto_service: CryptoService,
}


impl RouteService {
    fn new() -> RouteService {
        let wallet_service : WalletService =  WalletService::new();
        RouteService {
            wallet_service,
            crypto_service : CryptoService::new(),
        }
    }

    pub fn unpack_msg(&mut self, json_jwm: &str, my_vk: &str, wallet_handle: i32) -> Result<String, RouteError> {
        //check if jwm or jwm_compact
        let jwm_struct = match json_jwm.contains("recipients") {
            true => jwm::json_deserialize_jwm(json_jwm)?,
            false => jwm::deserialize_jwm_compact(json_jwm)?
        };

        let jwm_data = route_support::get_jwm_data(jwm_struct, my_vk)?;
        let my_key = route_support::get_key_from_str(my_vk, wallet_handle, &self.wallet_service)?;
        let sym_key = route_support::get_sym_key(&my_key, &jwm_data.cek, jwm_data.header, &self.crypto_service)?;
        //format payload to decrypt
        let payload = jwm_crypto::Payload {
            iv: jwm_data.iv,
            tag: jwm_data.tag,
            ciphertext: jwm_data.ciphertext,
            sym_key
        };

        //decrypt ciphertext
        jwm_crypto::decrypt_payload(&payload)
    }


    // This API call is made to encrypt both Application layer messages and Transport layer
// messages. The purpose of it is to take a message and wrap it up so that it can be fed into
// send_msg and on the other end unpack_msg can be called on it.
    pub fn pack_msg(&mut self, plaintext: &str, auth: bool, recv_keys: &Vec<String>, my_vk: Option<&str>, wallet_handle: i32) -> Result<String, RouteError> {
        //encrypt plaintext
        let encrypted_payload = jwm_crypto::encrypt_payload(plaintext);

        //convert str to Key
        let key = match my_vk {
            Some(vk) => Some(route_support::get_key_from_str(vk, wallet_handle, &self.wallet_service)?),
            None => None
        };

        //encrypt content_encryption_keys
        let encrypted_ceks = route_support::encrypt_ceks(recv_keys, auth, key, &encrypted_payload.sym_key, &self.crypto_service)?;

        //create jwm string
        match recv_keys.len() {
            //handles plaintext case
            0 => Err(RouteError::PackError("No receiving keys provided".to_string())),
            //handles single key case (compact serialization)
            1 => {
                jwm::serialize_jwm_compact(&recv_keys[0],
                                           &encrypted_ceks[0],
                                           my_vk,
                                           &encode(&encrypted_payload.ciphertext),
                                           &encode(&encrypted_payload.iv),
                                           &encode(&encrypted_payload.tag),
                                           auth)
            },
            //handles multi key case (JSON Serialization)
            _ => {
                jwm::json_serialize_jwm(&recv_keys,
                                        &encrypted_ceks,
                                        my_vk,
                                        &encode(&encrypted_payload.ciphertext),
                                        &encode(&encrypted_payload.iv),
                                        &encode(&encrypted_payload.tag),
                                        auth)
            }
        }
    }
}

//pub fn get_next_hop(did_with_key_frag: &str) -> (&str, &str) {
////DID#key is a reference identifier to the next hop
////their_vk is used to encrypt the message
////endpoint is the endpoint which the message is being sent to.
////called by send_msg()
////returns (endpoint, their_vk)
//
//}

#[cfg(test)]
pub mod tests {
    use services::wallet::WalletService;
    use services::crypto::CryptoService;
    use domain::crypto::key::{Key, KeyInfo};
    use domain::crypto::did::{Did, MyDidInfo, TheirDidInfo, TheirDid};
    use super::{RouteService};
    use std::collections::HashMap;
    use utils::inmem_wallet::InmemWallet;
    use utils::test::TestUtils;

    #[test]
    pub fn test_pack_msg_success_multi_anoncrypt(){
        _cleanup();
        //setup generic data to test
        let plaintext = "Hello World";
        let auth = false;

        let mut route_service = RouteService::new();

        //setup sendkey
        let (send_did, send_key) = _send_did1(&route_service.crypto_service);

        //setup receiving key
        let (recv_did, recv_key) = _recv_did2(&route_service.crypto_service);

        //create wallet
        route_service.wallet_service.create_wallet(&_config(), &_credentials());
        let wallet_handle = route_service.wallet_service.open_wallet(&_config(), &_credentials()).unwrap();

        //add send key and send DID
        route_service.wallet_service.add_indy_object(wallet_handle, &send_did.did, &send_did, &HashMap::new()).unwrap();
        route_service.wallet_service.add_indy_object(wallet_handle, &send_key.verkey, &send_key, &HashMap::new()).unwrap();

        //add receiving key and receiving DID
        route_service.wallet_service.add_indy_object(wallet_handle, &recv_did.did, &recv_did, &HashMap::new()).unwrap();
        route_service.wallet_service.add_indy_object(wallet_handle, &recv_key.verkey, &recv_key, &HashMap::new()).unwrap();

        //setup recv_keys to use with pack_msg
        let recv_keys = vec![_verkey2(&route_service.crypto_service),
                                        _verkey3(&route_service.crypto_service),
                                        _verkey4(&route_service.crypto_service)];

        //pack then unpack message
        let packed_msg = route_service.pack_msg(plaintext, auth, &recv_keys, None, wallet_handle).unwrap();
        let unpacked_msg = route_service.unpack_msg(&packed_msg, &recv_key.verkey, wallet_handle).unwrap();

        //verify same plaintext goes in and comes out
        assert_eq!(plaintext, &unpacked_msg);
    }

    #[test]
    pub fn test_unpack_msg_success_multi_anoncrypt() {
        _cleanup();

        let jwm = json!({
        "recipients":[
          {
             "header":{
                "typ":"x-b64nacl",
                "alg":"x-anon",
                "enc":"xsalsa20poly1305",
                "kid":"kqa2HyagzfMAq42H5f9u3UMwnSBPQx2QfrSyXbUPxMn"
             },
             "encrypted_key":"4uBqxCCVEJjkmPIu6gBVmv5LiXNVaMX3OLY_0vvjoyqsnwHztm_p8zDIGA30yXpgbGjOesqq0WwM3ACUZgkDk6yOXP_uNBrKIEIL9Lz_pJA="
          },
          {
             "header":{
                "typ":"x-b64nacl",
                "alg":"x-anon",
                "enc":"xsalsa20poly1305",
                "kid":"3SeuRm3uYuQDYmHeuMLu1xNHozNTtzS3kbZRFMMCWrX4"
             },
             "encrypted_key":"yOryGj8ZThWcrkt2NySjK2GG-BlMcaGmurZHeWBJqSleNZtEg7kav70tsH3NVaXuICvP6F53ur5kZvsV-2mFGSDTjVWyhWkS7KymNnc4TEU="
          },
          {
             "header":{
                "typ":"x-b64nacl",
                "alg":"x-anon",
                "enc":"xsalsa20poly1305",
                "kid":"Dqc95QYYCot8XNLp9APubEP7omDqHHVU9frwFSUb9yBu"
             },
             "encrypted_key":"hqPDEQMfHjdpc_4HK77D4twsgWw5h8nPjbOf72bodDRDLTU5OZHwBr5QvAnRoeVS8pN2m8Mmm4hY4FTgbVaao7oh5iVUh3Z31boL29dQiy0="
          }
       ],
       "ciphertext":"L4AhvoZpb_-t-Ig=",
       "iv":"ZtXIyB8PTI5QrCQJ-XWhxoqvm9K3TdS3",
       "tag":"DmpIMQSeqeyKtgdBJGt-9w=="}).to_string();

        let mut route_service : RouteService = RouteService::new();

        let (did, key) = _recv_did2(&route_service.crypto_service);

        route_service.wallet_service.create_wallet(&_config(), &_credentials());
        let wallet_handle = route_service.wallet_service.open_wallet(&_config(), &_credentials()).unwrap();
        route_service.wallet_service.add_indy_object(wallet_handle, &did.did, &did, &HashMap::new()).unwrap();
        route_service.wallet_service.add_indy_object(wallet_handle, &key.verkey, &key, &HashMap::new()).unwrap();

        let plaintext = route_service.unpack_msg(&jwm, &key.verkey, wallet_handle).unwrap();
        assert_eq!(plaintext, "Hello World".to_string());
    }

    fn _send_did1(service : &CryptoService) -> (Did, Key) {
        let did_info = MyDidInfo { did: None, cid: None, seed: Some("00000000000000000000000000000My1".to_string()), crypto_type: None };
        service.create_my_did(&did_info).unwrap()
    }

    fn _recv_did2(service : &CryptoService) -> (Did, Key) {
        let did_info = MyDidInfo { did: None, cid: None, seed: Some("00000000000000000000000000000My2".to_string()), crypto_type: None };
        service.create_my_did(&did_info).unwrap()
    }

    fn _verkey2(service : &CryptoService) -> String {
        let did_info = MyDidInfo { did: None, cid: None, seed: Some("00000000000000000000000000000My2".to_string()), crypto_type: None };
        let (_, key) = service.create_my_did(&did_info).unwrap();
        key.verkey
    }

    fn _verkey3(service : &CryptoService) -> String {
        let did_info = MyDidInfo { did: None, cid: None, seed: Some("00000000000000000000000000000My3".to_string()), crypto_type: None };
        let (_, key) = service.create_my_did(&did_info).unwrap();
        key.verkey
    }

    fn _verkey4(service : &CryptoService) -> String {
        let did_info = MyDidInfo { did: None, cid: None, seed: Some("00000000000000000000000000000My4".to_string()), crypto_type: None };
        let (_, key) = service.create_my_did(&did_info).unwrap();
        key.verkey
    }

    fn _config() -> String {
        json!({"id": "w1"}).to_string()
    }

    fn _credentials() -> String {
        json!({"key": "my_key"}).to_string()
    }

    fn _cleanup() {
        TestUtils::cleanup_storage();
        InmemWallet::cleanup();
    }
}