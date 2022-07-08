use rand::Rng;

use crate::pki::{SecretKey, PublicKey, Signature};

#[derive(Debug)]
pub struct Challenge([u8; 32]);

#[derive(Debug)]
pub struct ChallengeParseError;

impl Challenge {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut arr = [0; 32];
        rng.fill(&mut arr[..]);
        Self(arr)
    }

    pub fn sign(&self, secret_key: &SecretKey) -> Signature {
        secret_key.sign(&self.0)
    }


    pub fn verify(&self, public_key: &PublicKey, signature: &Signature) -> bool {
        public_key.verify(&self.0, signature)
    }

    pub fn bytes(&self) -> [u8; 32] {
        self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ChallengeParseError> {
        bytes.try_into().map_err(|_| ChallengeParseError {}).map(Self)
    } 
}

impl Default for Challenge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    extern crate wasm_bindgen_test;

    use wasm_bindgen_test::*;

    use crate::pki::SecretKey;
    use super::*;

    #[wasm_bindgen_test]
    fn test_handshake_success() {
        let secret = SecretKey::generate();
        let public = secret.public_key();

        let challenge = Challenge::new();
        let sig = challenge.sign(&secret);
        let verified = challenge.verify(&public, &sig);

        assert!(verified);
    }

    #[wasm_bindgen_test]
    fn test_handshake_fail() {
        let secret = SecretKey::generate();
        let public = SecretKey::generate().public_key(); // public key from a different private key

        let challenge = Challenge::new();
        let sig = challenge.sign(&secret);
        let verified = challenge.verify(&public, &sig);

        assert!(!verified);
    }
}
