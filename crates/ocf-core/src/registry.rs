use crate::codec::{Codec, CodecId};
use crate::transform::{Transform, TransformId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct CodecRegistry {
    codecs: RwLock<HashMap<CodecId, Arc<dyn Codec>>>,
}

impl CodecRegistry {
    pub fn new() -> Self { Self::default() }
    pub fn register(&self, c: Arc<dyn Codec>) { self.codecs.write().unwrap().insert(c.id(), c); }
    pub fn get(&self, id: CodecId) -> Option<Arc<dyn Codec>> { self.codecs.read().unwrap().get(&id).cloned() }
    pub fn contains(&self, id: CodecId) -> bool { self.codecs.read().unwrap().contains_key(&id) }
    pub fn ids(&self) -> Vec<CodecId> { self.codecs.read().unwrap().keys().cloned().collect() }
    pub fn len(&self) -> usize { self.codecs.read().unwrap().len() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
}

#[derive(Default)]
pub struct TransformRegistry {
    transforms: RwLock<HashMap<TransformId, Arc<dyn Transform>>>,
}

impl TransformRegistry {
    pub fn new() -> Self { Self::default() }
    pub fn register(&self, t: Arc<dyn Transform>) { self.transforms.write().unwrap().insert(t.id(), t); }
    pub fn get(&self, id: TransformId) -> Option<Arc<dyn Transform>> { self.transforms.read().unwrap().get(&id).cloned() }
    pub fn contains(&self, id: TransformId) -> bool { self.transforms.read().unwrap().contains_key(&id) }
    pub fn len(&self) -> usize { self.transforms.read().unwrap().len() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
}
