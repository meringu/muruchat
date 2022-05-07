use dioxus::prelude::*;
// use serde::de::{Deserializer, MapAccess, Visitor};
// use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Iter, HashMap};

use crate::pki::PublicKey;

pub static ADDRESS_BOOK: Atom<AddressBook> = |_| AddressBook::from_context();

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AddressBook {
    contacts: HashMap<PublicKey, String>,
}

// impl Serialize for AddressBook {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut map = serializer.serialize_map(Some(self.contacts.len()))?;
//         for (public_key, nickname) in &self.contacts {
//             let der = public_key.to_public_key_der().unwrap();
//             map.serialize_entry(&base64::encode(der), &nickname)?;
//         }
//         map.end()
//     }
// }

// struct AddressBookVisitor {
//     marker: PhantomData<fn() -> HashSet<RsaPublicKey>>,
// }

// impl AddressBookVisitor {
//     fn new() -> Self {
//         AddressBookVisitor {
//             marker: PhantomData,
//         }
//     }
// }

// impl<'de> Visitor<'de> for AddressBookVisitor {
//     type Value = AddressBook;

//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         formatter.write_str("address book")
//     }

//     fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
//     where
//         A: MapAccess<'de>,
//     {
//         let mut contacts = HashMap::with_capacity(map.size_hint().unwrap_or(0));

//         while let Some((der, nickname)) = map.next_entry()? {
//             let s: String = der;
//             let public_key =
//                 RsaPublicKey::from_public_key_der(&base64::decode(s).unwrap()).unwrap();

//             contacts.insert(public_key, nickname);
//         }

//         Ok(AddressBook { contacts })
//     }
// }

// impl<'de> Deserialize<'de> for AddressBook {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         deserializer.deserialize_map(AddressBookVisitor::new())
//     }
// }

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
        public_key: PublicKey,
    ) -> Result<(), String> {
        if nickname == "" {
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
