use der::Document;
use dioxus::prelude::*;
use pem_rfc7468::LineEnding;
use pkcs8::{spki::EncodePublicKey, DecodePrivateKey, EncodePrivateKey};
use rsa::{RsaPrivateKey, RsaPublicKey};

pub static USER: Atom<Option<User>> = |_| User::from_context();

#[derive(Clone)]
pub struct User {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

impl User {
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let bits = 2048;
        let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        Self::from_private_key(private_key)
    }

    pub fn from_context() -> Option<Self> {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        match storage.get_item("private_key").unwrap() {
            Some(private_key_pem) => {
                let private_key = RsaPrivateKey::from_pkcs8_pem(&private_key_pem).unwrap();
                Some(Self::from_private_key(private_key))
            }
            None => None,
        }
    }

    pub fn from_private_key(private_key: RsaPrivateKey) -> Self {
        let public_key = RsaPublicKey::from(&private_key);

        Self {
            private_key,
            public_key,
        }
    }

    pub fn delete(&self) {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        storage.remove_item("private_key").unwrap()
    }

    pub fn private_key(&self) -> RsaPrivateKey {
        self.private_key.clone()
    }

    pub fn private_key_pem(&self) -> String {
        self.private_key
            .to_pkcs8_der()
            .unwrap()
            .to_pem(LineEnding::LF)
            .unwrap()
            .trim()
            .to_string()
    }

    pub fn public_key(&self) -> RsaPublicKey {
        self.public_key.clone()
    }

    pub fn public_key_pem(&self) -> String {
        self.public_key
            .to_public_key_der()
            .unwrap()
            .to_pem(LineEnding::LF)
            .unwrap()
            .trim()
            .to_string()
    }

    pub fn save(&self) {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        let pem = self
            .private_key
            .to_pkcs8_der()
            .unwrap()
            .to_pem(LineEnding::LF)
            .unwrap();
        storage.set("private_key", &pem).unwrap();
    }
}
