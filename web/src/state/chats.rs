use dioxus::prelude::*;
// use serde::de::{Deserializer, SeqAccess, Visitor};
// use serde::ser::{SerializeSeq, Serializer};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map, hash_set, HashMap, HashSet};

use crate::pki::PublicKey;

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

    pub fn iter(&self) -> hash_map::Iter<String, Chat> {
        self.chats.iter()
    }

    pub fn any(&self) -> bool {
        self.len() > 0
    }

    pub fn none(&self) -> bool {
        !self.any()
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Chat {
    peers: HashSet<PublicKey>,
    id: String,
}

impl Chat {
    pub fn new(peers: HashSet<PublicKey>) -> Self {
        let mut id = [0; 33];

        for public_key in peers.iter() {
            for (i, byte) in public_key.bytes().iter().enumerate() {
                id[i] = id[i] ^ byte
            }
        }

        Self {
            peers,
            id: hex::encode(id),
        }
    }

    pub fn from_public_key(public_key: PublicKey) -> Self {
        let mut peers = HashSet::new();
        peers.insert(public_key);
        Self::new(peers)
    }

    pub fn from_public_keys(public_keys: Vec<PublicKey>) -> Self {
        let mut peers = HashSet::new();
        for public_key in public_keys {
            peers.insert(public_key);
        }
        Self::new(peers)
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn iter(&self) -> hash_set::Iter<PublicKey> {
        self.peers.iter()
    }

    pub fn group_chat_with(&self, public_key: PublicKey) -> Self {
        let mut peers = self.peers.clone();
        peers.insert(public_key);
        Self::new(peers)
    }
}
