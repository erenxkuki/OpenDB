use crate::content_address::{ContentId, ContentStore};

#[derive(Debug, Clone)]
pub struct Dictionary {
    pub id: ContentId,
    pub bytes: Vec<u8>,
}

#[derive(Default)]
pub struct DictionaryStore {
    store: ContentStore,
}

impl DictionaryStore {
    pub fn new() -> Self { Self::default() }
    pub fn add(&mut self, bytes: Vec<u8>) -> ContentId {
        self.store.put(&bytes)
    }
    pub fn get(&self, id: &ContentId) -> Option<&[u8]> { self.store.get(id) }
    pub fn contains(&self, id: &ContentId) -> bool { self.store.contains(id) }
    pub fn len(&self) -> usize { self.store.len() }
}
