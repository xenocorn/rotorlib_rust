use crate::protocol::{Key, hash};

use std::collections::HashMap;
use std::collections::hash_map::Values;

#[derive(Debug, Clone)]
pub struct Session{
    is_self_a_node: bool,
    subscriptions: HashMap<String, Key>,
}

impl Session{
    pub fn new() -> Self{
        Self{is_self_a_node: false, subscriptions: HashMap::new()}
    }
    pub fn set_a_node(&mut self, node: bool) -> Option<()>{
        match node != self.is_self_a_node{
            true => {
                self.is_self_a_node = node;
                Some(())
            }
            false => { None }
        }
    }
    pub fn is_a_node(&self) -> bool{ self.is_self_a_node }
    pub fn clear_subscriptions(&mut self) -> &Self{
        self.subscriptions = HashMap::new();
        self
    }
    pub fn sub(&mut self, key: String) -> Option<Key>{
        if self.subscriptions.contains_key(&key){
            return None;
        }
        let int_key = hash(&key);
        self.subscriptions.insert(key, int_key);
        Some(int_key)
    }
    pub fn unsub(&mut self, key: String) -> Option<Key>{
        match self.subscriptions.remove(&key) {
            None => {None}
            Some(int_key) => {Some(int_key)}
        }
    }
    pub fn is_sub(&self, key: String) -> Option<Key>{
        match self.subscriptions.get(&key){
            None => { None }
            Some(k) => { Some(*k) }
        }
    }
    pub fn sub_keys(&self) -> Values<String, u32>{
        self.subscriptions.values()
    }
}
