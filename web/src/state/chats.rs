use dioxus::prelude::*;
use pkcs8::spki::EncodePublicKey;
use rsa::RsaPublicKey;
use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, Serializer};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{hash_map::Iter, HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;

pub static CHATS: Atom<Chats> = |_| Chats::from_context();

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Chats {
    chats: HashMap<String, Chat>,
}

impl Chats {
    pub fn from_context() -> Self {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        match storage.get_item("chats").unwrap() {
            Some(c) => serde_json::from_str(&c).unwrap(),
            None => Self::default(),
        }
    }

    pub fn add_chat(&mut self, id: String, chat: Chat) -> Result<(), String> {
        self.chats.insert(id, chat);

        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<Chat> {
        self.chats.get(id).cloned()
    }

    pub fn iter(&self) -> Iter<String, Chat> {
        self.chats.iter()
    }

    pub fn len(&self) -> usize {
        self.chats.len()
    }

    pub fn save(&self) {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        storage
            .set("chats", &serde_json::to_string(&self).unwrap())
            .unwrap();
    }

    pub fn delete(&self) {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        storage.delete("chats").unwrap();
    }
}

#[derive(Clone)]
pub struct Chat {
    peers: HashSet<RsaPublicKey>,
    id: String,
}

impl Serialize for Chat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.peers.len()))?;
        for public_key in self.peers.iter() {
            seq.serialize_element(&public_key)?;
        }
        seq.end()
    }
}

struct ChatVisitor {
    marker: PhantomData<fn() -> HashSet<RsaPublicKey>>,
}

impl ChatVisitor {
    fn new() -> Self {
        ChatVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for ChatVisitor {
    type Value = Chat;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("chat")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut peers = HashSet::with_capacity(seq.size_hint().unwrap_or(0));

        while let Some(public_key) = seq.next_element()? {
            peers.insert(public_key);
        }

        Ok(Chat::new(peers))
    }
}

impl<'de> Deserialize<'de> for Chat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ChatVisitor::new())
    }
}

impl Chat {
    pub fn new(peers: HashSet<RsaPublicKey>) -> Self {
        let mut id = [0; 32];

        for public_key in peers.iter() {
            let mut hasher = Sha256::new();
            let bytes = public_key.to_public_key_der().unwrap();
            hasher.update(bytes);
            let result = hasher.finalize();

            for (i, byte) in result.iter().enumerate() {
                id[i] = id[i] ^ byte
            }
        }

        Self {
            peers,
            id: hex::encode(id),
        }
    }

    pub fn from_public_key(public_key: RsaPublicKey) -> Self {
        let mut peers = HashSet::new();
        peers.insert(public_key);
        Self::new(peers)
    }

    pub fn from_public_keys(public_keys: Vec<RsaPublicKey>) -> Self {
        let mut peers = HashSet::new();
        for public_key in public_keys {
            peers.insert(public_key);
        }
        Self::new(peers)
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn group_chat_with(&self, public_key: RsaPublicKey) -> Self {
        let mut peers = self.peers.clone();
        peers.insert(public_key);
        Self::new(peers)
    }
}
