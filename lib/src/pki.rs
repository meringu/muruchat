use k256::{elliptic_curve::sec1::ToEncodedPoint, ecdsa::{self, SigningKey, VerifyingKey, signature::{Signer, Verifier}}};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    str::FromStr,
};

#[derive(Debug, Clone, std::cmp::Eq)]
pub struct PublicKey(k256::PublicKey);

#[derive(PartialEq, Clone)]
pub struct SecretKey(k256::SecretKey);

#[derive(Debug)]
pub struct Signature(ecdsa::Signature);

struct SignatureVisitor;

impl<'de> Visitor<'de> for SignatureVisitor {
    type Value = Signature;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a secp256k1 signature")
    }

    fn visit_str<E>(self, value: &str) -> Result<Signature, E>
    where
        E: de::Error,
    {
        let bytes = hex::decode(value).map_err(|e| de::Error::custom(e))?;
        let sig = Signature::from_bytes(&bytes).map_err(|e| de::Error::custom(e))?;

        Ok(sig)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de> {
        deserializer.deserialize_str(SignatureVisitor)
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        // Default implementation just delegates to `deserialize` impl.
        *place = Deserialize::deserialize(deserializer)?;
        Ok(())
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer {
        serializer.collect_str(&hex::encode(self.bytes()))
    }
}

#[derive(Debug)]
pub struct PublicKeyParseError;

#[derive(Debug)]
pub struct SecretKeyParseError;

#[derive(Debug)]
pub struct SignatureParseError;

impl Display for SignatureParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to parse signature")
    }
}

struct PublicKeyVisitor;

impl PublicKey {
    pub fn bytes(&self) -> [u8; 33] {
        self.0.to_encoded_point(true).as_bytes().try_into().unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PublicKeyParseError> {
        k256::PublicKey::from_sec1_bytes(
            bytes
        ).map_err(|_| PublicKeyParseError {}).map(Self)
    }

    pub fn verify(&self, bytes: &[u8], signature: &Signature) -> bool {
        VerifyingKey::from(&self.0).verify(bytes, &signature.0).is_ok()
    }
}

impl SecretKey {
    pub fn generate() -> Self {
        let rng = rand::thread_rng();
        let secret_key = k256::SecretKey::random(rng);
        Self(secret_key)
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.public_key())
    }

    pub fn sign(&self, bytes: &[u8]) -> Signature {
        let sig: ecdsa::Signature = SigningKey::from(&self.0).sign(bytes);

        Signature(sig)
    }
}

impl Signature {
    pub fn bytes(&self) -> &[u8] {
        use k256::ecdsa::signature::Signature;

        self.0.as_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SignatureParseError> {
        ecdsa::signature::Signature::from_bytes(
            bytes
        ).map_err(|_| SignatureParseError {}).map(Self)
    }
}

impl PartialEq for PublicKey {
    fn eq(&self, other: &Self) -> bool { 
        self.bytes() == other.bytes()
    }
}


impl Hash for PublicKey {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        state.write(&self.bytes());
        state.finish();
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.bytes()))
    }
}

impl fmt::Display for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded = hex::encode(self.0.to_be_bytes());

        write!(f, "{}", encoded)
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PublicKey {
    type Err = PublicKeyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = hex::decode(s).map_err(|_| Self::Err {})?;

        Self::from_bytes(&decoded)
    }
}

impl FromStr for SecretKey {
    type Err = SecretKeyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = hex::decode(s).map_err(|_| Self::Err {})?;

        k256::SecretKey::from_be_bytes(&decoded)
            .map_err(|_| Self::Err {})
            .map(Self)
    }
}

impl FromStr for Signature {
    type Err = SignatureParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ecdsa::Signature::from_str(s)
            .map_err(|_| Self::Err {})
            .map(Self)
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Serialize for SecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Visitor<'de> for PublicKeyVisitor {
    type Value = PublicKey;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("secp256k1 public key")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        PublicKey::from_str(value)
            .map_err(|_| E::custom(format!("failed to parse public key: {}", value)))
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PublicKeyVisitor)
    }
}

#[cfg(test)]
mod tests {
    extern crate wasm_bindgen_test;

    use wasm_bindgen_test::*;

    use super::*;

    #[wasm_bindgen_test]
    fn test_public_key_length() {
        let secret = SecretKey::generate();
        let public = secret.public_key();

        assert_eq!(public.bytes().len(), 33);
    }

    #[wasm_bindgen_test]
    fn test_serialize_deserialize() {
        let secret = SecretKey::generate();
        let public = secret.public_key();

        let ser = serde_json::to_string(&public).unwrap();
        let de_ser: PublicKey = serde_json::from_str(&ser).unwrap();

        assert_eq!(de_ser, public);
    }
}
