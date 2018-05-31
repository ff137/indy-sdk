use std::rc::Rc;

use errors::wallet::WalletError;

use super::WalletRecord;
use super::wallet::Keys;
use super::storage::StorageIterator;
use super::encryption::{decrypt, decrypt_tags};


pub(super) struct WalletIterator {
    storage_iterator: Box<StorageIterator>,
    keys: Rc<Keys>,
}


impl WalletIterator {
    pub fn new(storage_iter: Box<StorageIterator>, keys: Rc<Keys>) -> Self {
        WalletIterator {
            storage_iterator: storage_iter,
            keys: keys,
        }
    }

    pub fn next(&mut self) -> Result<Option<WalletRecord>, WalletError> {
        let next_storage_entity = self.storage_iterator.next()?;
        if let Some(next_storage_entity) = next_storage_entity {
            let decrypted_name = decrypt(&next_storage_entity.name, &self.keys.name_key)?;
            let name = String::from_utf8(decrypted_name)?;

            let value = match next_storage_entity.value {
                None => None,
                Some(encrypted_value) => Some(encrypted_value.decrypt(&self.keys.value_key)?)
            };

            let tags = decrypt_tags(&next_storage_entity.tags, &self.keys.tag_name_key, &self.keys.tag_value_key)?;

            Ok(Some(WalletRecord::new(name, None, value, tags)))
        } else { Ok(None) }
    }
}