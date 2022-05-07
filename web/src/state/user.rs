use dioxus::prelude::*;
use std::str::FromStr;

use muruchat::pki::{PublicKey, SecretKey};

pub static USER: Atom<Option<User>> = |_| User::from_context();

#[derive(Clone)]
pub struct User {
    secret_key: SecretKey,
    public_key: PublicKey,
}

impl User {
    pub fn generate() -> Self {
        Self::from_secret_key(SecretKey::generate())
    }

    pub fn from_context() -> Option<Self> {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        storage.get_item("secret_key").unwrap().map(|secret_key|
            Self::from_secret_key(
                SecretKey::from_str(&secret_key).unwrap(),
            )
        )
    }

    pub fn from_secret_key(secret_key: SecretKey) -> Self {
        let public_key = secret_key.public_key();

        Self {
            secret_key,
            public_key,
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn secret_key(&self) -> SecretKey {
        self.secret_key.clone()
    }

    pub fn delete(&self) {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        storage.remove_item("private_key").unwrap()
    }

    pub fn save(&self) {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        let doc = self.secret_key.to_string();
        storage.set("secret_key", &doc).unwrap();
    }
}
