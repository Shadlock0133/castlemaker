use serde::{Serialize, Deserialize};
use std::{
    cmp::Eq,
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut},
};

pub trait Counter {
    fn new() -> Self;
    fn inc(&mut self) -> Self;
}

impl Counter for usize {
    fn new() -> Self { 0 }
    fn inc(&mut self) -> Self { let ret = *self; *self += 1; ret }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CounterMap<K, V> where K: Eq + Hash {
    hashmap: HashMap<K, V>,
    counter: K,
}

impl<K: Counter + Eq + Hash + Clone, V> CounterMap<K, V> {
    pub fn new() -> Self {
        Self {
            hashmap: HashMap::new(),
            counter: K::new(),
        }
    }

    pub fn push(&mut self, value: V) -> Option<K> {
        let key = self.counter.inc();
        self.hashmap.insert(key.clone(), value).map(|_| key)
    }
}

impl<K: Eq + Hash, V> Deref for CounterMap<K, V> {
    type Target = HashMap<K, V>;
    fn deref(&self) -> &Self::Target {
        &self.hashmap
    }
}

impl<K: Eq + Hash, V> DerefMut for CounterMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hashmap
    }
}