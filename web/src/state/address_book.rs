use dioxus::prelude::*;
// use serde::de::{Deserializer, MapAccess, Visitor};
// use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Iter, HashMap};

use muruchat::pki::PublicKey;

pub static ADDRESS_BOOK: Atom<AddressBook> = |_| AddressBook::from_context();

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AddressBook {
    contacts: HashMap<PublicKey, String>,
}

impl AddressBook {
    pub fn from_context() -> Self {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        match storage.get_item("address_book").unwrap() {
            Some(book) => serde_json::from_str(&book).unwrap(),
            None => Self::default(),
        }
    }

    pub fn add_contact(&mut self, nickname: String, public_key: PublicKey) -> Result<(), String> {
        if nickname.is_empty() {
            return Err("Invalid nickname.".to_string());
        }

        for (_, other) in self.contacts.iter() {
            if *other == nickname {
                return Err("A contact with that nickname already exists.".to_string());
            }
        }
        if self.contacts.contains_key(&public_key) {
            return Err("A contact with that public already exists.".to_string());
        }

        self.contacts.insert(public_key, nickname);

        Ok(())
    }

    pub fn iter(&self) -> Iter<PublicKey, String> {
        self.contacts.iter()
    }

    pub fn who_is(&self, public_key: &PublicKey) -> Option<String> {
        self.contacts.get(public_key).cloned()
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
