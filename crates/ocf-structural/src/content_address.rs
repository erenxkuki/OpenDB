use ocf_core::content_hash;
use std::collections::HashMap;

pub type ContentId = [u8; 32];

#[derive(Default)]
pub struct ContentStore {
    objects: HashMap<ContentId, Vec<u8>>,
}

impl ContentStore {
    pub fn new() -> Self { Self::default() }
    pub fn put(&mut self, bytes: &[u8]) -> ContentId {
        let id = content_hash(bytes);
        self.objects.entry(id).or_insert_with(|| bytes.to_vec());
        id
    }
    pub fn get(&self, id: &ContentId) -> Option<&[u8]> {
        self.objects.get(id).map(|v| v.as_slice())
    }
    pub fn contains(&self, id: &ContentId) -> bool { self.objects.contains_key(id) }
    pub fn len(&self) -> usize { self.objects.len() }
    pub fn is_empty(&self) -> bool { self.objects.is_empty() }
    pub fn remove_if_unreferenced(&mut self, id: &ContentId, refcount: usize) -> bool {
        if refcount == 0 { self.objects.remove(id).is_some() } else { false }
    }
}

pub fn id_from_hash(hash: &[u8; 32]) -> u128 {
    u128::from_le_bytes(hash[..16].try_into().unwrap())
}
