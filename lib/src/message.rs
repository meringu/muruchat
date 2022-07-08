use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::pki::{PublicKey, Signature, SecretKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub to: PublicKey,
    pub from: PublicKey,
    ciphertext: Vec<u8>,
    signature: Signature,    
}

impl Message {
    pub fn new(to: &PublicKey, secret_key: &SecretKey, plaintext: &str)-> Self {
        // TODO encrypt this
        let ciphertext = plaintext.as_bytes().to_vec();

        let from = secret_key.public_key();

        let signature = secret_key.sign(&Self::sig_material(to, &from, &ciphertext));

        Self {
            to: to.clone(),
            from,
            ciphertext,
            signature
        }
    }

    pub fn verify(&self) -> bool {
        self.from.verify( &Self::sig_material(&self.to, &self.from, &self.ciphertext), &self.signature)
    }

    pub fn decrypt(&self) -> Result<String, Box<dyn Error>> {
        Ok(std::str::from_utf8(&self.ciphertext)?.to_string())
    }

    fn sig_material(to: &PublicKey, from: &PublicKey, ciphertext: &[u8]) -> Vec<u8> {
        return [
            to.bytes().as_slice(),
            from.bytes().as_slice(),
            ciphertext,
        ].concat()
    }
}

#[cfg(test)]
mod tests {
    extern crate wasm_bindgen_test;

    use wasm_bindgen_test::*;

    use super::*;

    #[wasm_bindgen_test]
    fn test_message_verify() {
        let to_public =  SecretKey::generate().public_key();
        let from_secret = SecretKey::generate();

        let plaintext = "The quick brown fox jumps over the lazy dog";

        let message = Message::new(&to_public, &from_secret, plaintext);

        let verified = message.verify();

        assert!(verified);
    }

    #[wasm_bindgen_test]
    fn test_message_decrypt() {
        let to_public =  SecretKey::generate().public_key();
        let from_secret = SecretKey::generate();

        let plaintext = "The quick brown fox jumps over the lazy dog";

        let message = Message::new(&to_public, &from_secret, plaintext);

        let decrypted = message.decrypt().unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
