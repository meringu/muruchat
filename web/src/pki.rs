use k256::elliptic_curve::sec1::ToEncodedPoint;
use std::{fmt, str::FromStr, hash::{Hash, Hasher}};
use serde::{de::{self, Visitor}, Serialize, Serializer, Deserialize, Deserializer};

#[derive(PartialEq, Clone, std::cmp::Eq)]
pub struct PublicKey(k256::PublicKey);

#[derive(PartialEq, Clone)]
pub struct SecretKey(k256::SecretKey);

#[derive(Debug)]
pub struct PublicKeyParseError;

#[derive(Debug)]
pub struct SecretKeyParseError;

struct PublicKeyVisitor;

impl PublicKey {
    pub fn bytes(&self) -> [u8; 33] {
        self.0.to_encoded_point(true).as_bytes().try_into().unwrap()
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
}

impl Hash for PublicKey {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher
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

impl FromStr for PublicKey {
    type Err = PublicKeyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = hex::decode(s).map_err(|_| Self::Err {})?;

        k256::PublicKey::from_sec1_bytes(&decoded)
            .map_err(|_| Self::Err {})
            .map(|inner| Self(inner))
    }
}

impl FromStr for SecretKey {
    type Err = SecretKeyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = hex::decode(s).map_err(|_| Self::Err {})?;

        k256::SecretKey::from_be_bytes(&decoded)
            .map_err(|_| Self::Err {})
            .map(|inner| Self(inner))
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
        PublicKey::from_str(value).map_err(|_| E::custom(format!("failed to parse public key: {}", value)))
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

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_length() {
        let secret = SecretKey::generate();
        let public = secret.public_key();

        assert_eq!(public.to_string().len(), 33);
    }
}
