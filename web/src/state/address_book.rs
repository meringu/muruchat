use dioxus::prelude::*;
use pkcs8::spki::EncodePublicKey;
use rsa::RsaPublicKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{hash_map::Iter, HashMap, HashSet};

pub static ADDRESS_BOOK: Atom<AddressBook> = |_| AddressBook::from_context();

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AddressBook {
    contacts: HashMap<String, Contact>,
}

impl AddressBook {
    pub fn from_context() -> Self {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        match storage.get_item("address_book").unwrap() {
            Some(book) => serde_json::from_str(&book).unwrap(),
            None => Self::default(),
        }
    }

    pub fn add_contact(
        &mut self,
        nickname: String,
        public_key: RsaPublicKey,
    ) -> Result<(), String> {
        if nickname == "" {
            return Err("Invalid nickname.".to_string());
        }

        for (_, contact) in self.contacts.iter() {
            if contact.public_key() == public_key {
                return Err("A contact with that public key already exists.".to_string());
            }
        }
        if self.contacts.contains_key(&nickname) {
            return Err("A contact with that nickname already exists.".to_string());
        }

        self.contacts
            .insert(nickname, Contact::from_public_key(public_key));

        Ok(())
    }

    pub fn iter(&self) -> Iter<String, Contact> {
        self.contacts.iter()
    }

    pub fn len(&self) -> usize {
        self.contacts.len()
    }

    pub fn save(&self) {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        storage
            .set("address_book", &serde_json::to_string(&self).unwrap())
            .unwrap();
    }

    pub fn delete(&self) {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        storage.delete("address_book").unwrap();
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Contact(RsaPublicKey);

impl Contact {
    fn from_public_key(public_key: RsaPublicKey) -> Self {
        Self(public_key)
    }

    pub fn public_key(&self) -> RsaPublicKey {
        self.0.clone()
    }

    pub fn get_chat(&self) -> Chat {
        Chat::from_public_key(self.0.clone())
    }
}

pub struct Chat(HashSet<RsaPublicKey>);

impl Chat {
    fn hash_public_key(public_key: &RsaPublicKey) -> [u8; 32] {
        let mut hasher = Sha256::new();
        let bytes = public_key.to_public_key_der().unwrap();
        hasher.update(bytes);
        let result = hasher.finalize();
        result.as_slice().try_into().unwrap()
    }

    pub fn from_public_key(public_key: RsaPublicKey) -> Self {
        let mut set = HashSet::new();
        set.insert(public_key);
        Self(set)
    }

    pub fn from_public_keys(public_keys: Vec<RsaPublicKey>) -> Self {
        let mut set = HashSet::new();
        for public_key in public_keys {
            set.insert(public_key);
        }
        Self(set)
    }

    pub fn id(&self) -> String {
        let mut id = [0; 32];
        for public_key in self.0.iter() {
            for (i, byte) in Self::hash_public_key(public_key).iter().enumerate() {
                id[i] = id[i] ^ byte
            }
        }
        hex::encode(id)
    }

    pub fn group_chat_with(&self, public_key: RsaPublicKey) -> Self {
        let mut set = self.0.clone();
        set.insert(public_key);
        Self(set)
    }
}
