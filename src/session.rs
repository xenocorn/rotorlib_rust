use crate::protocol::{Key, get_route_key};

use std::collections::HashMap;
use std::collections::hash_map::{Values, Keys, IntoIter};

#[derive(Debug, Clone)]
pub struct Session{
    is_self_a_router: bool,
    subscriptions: HashMap<String, Key>,
}

impl Session{
    pub fn new() -> Self{
        Self{ is_self_a_router: false, subscriptions: HashMap::new()}
    }
    pub fn set_a_router(&mut self, is_a_router: bool) -> Option<()>{
        match is_a_router != self.is_self_a_router {
            true => {
                self.is_self_a_router = is_a_router;
                Some(())
            }
            false => { None }
        }
    }
    pub fn is_a_router(&self) -> bool{ self.is_self_a_router }
    pub fn clear_subscriptions(&mut self) -> &Self{
        self.subscriptions = HashMap::new();
        self
    }
    pub fn sub(&mut self, key: String) -> Option<Key>{
        if self.subscriptions.contains_key(&key){
            return None;
        }
        let int_key = get_route_key(&key);
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
    pub fn sub_router_keys(&self) -> Values<String, Key>{
        self.subscriptions.values()
    }
    pub fn sub_msg_keys(&self) -> Keys<String, Key>{
        self.subscriptions.keys()
    }
    pub fn all_keys(&self) -> IntoIter<String, u64> {
        self.subscriptions.clone().into_iter()
    }
}
