use std::collections::{HashMap, VecDeque};

pub struct Cache {
    capacity_bytes: u64,
    used_bytes: u64,
    map: HashMap<u64, (Vec<u8>, usize)>, // key -> (data, size index)
    keys_order: VecDeque<u64>,
}

impl Cache {
    pub fn new(capacity_bytes: u64) -> Self {
        Self { capacity_bytes, used_bytes: 0, map: HashMap::new(), keys_order: VecDeque::new() }
    }
    pub fn get(&self, key: u64) -> Option<&[u8]> {
        self.map.get(&key).map(|(d, _)| d.as_slice())
    }
    pub fn insert(&mut self, key: u64, data: Vec<u8>) {
        let sz = data.len() as u64;
        if sz > self.capacity_bytes { return; }
        // evict
        while self.used_bytes + sz > self.capacity_bytes {
            if let Some(old) = self.keys_order.pop_front() {
                if let Some((_, s)) = self.map.remove(&old) {
                    self.used_bytes = self.used_bytes.saturating_sub(s as u64);
                }
            } else { break; }
        }
        self.used_bytes += sz;
        self.map.insert(key, (data, sz as usize));
        self.keys_order.push_back(key);
    }
    pub fn used_bytes(&self) -> u64 { self.used_bytes }
    pub fn capacity_bytes(&self) -> u64 { self.capacity_bytes }
    pub fn len(&self) -> usize { self.map.len() }
}
