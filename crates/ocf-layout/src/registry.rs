use crate::layout::PhysicalLayout;

#[derive(Default)]
pub struct LayoutRegistry {
    pub layouts: Vec<PhysicalLayout>,
    pub current: Option<u128>,
    pub previous: Option<u128>,
}

impl LayoutRegistry {
    pub fn new() -> Self { Self::default() }
    pub fn register(&mut self, l: PhysicalLayout) {
        self.layouts.push(l);
    }
    pub fn get(&self, id: u128) -> Option<&PhysicalLayout> {
        self.layouts.iter().find(|l| l.layout_id == id)
    }
    pub fn current_layout(&self) -> Option<&PhysicalLayout> {
        self.current.and_then(|id| self.get(id))
    }
    /// Atomic publish: set current to candidate, keep previous. Requires candidate exists.
    pub fn publish(&mut self, candidate_id: u128) -> bool {
        if self.get(candidate_id).is_none() { return false; }
        self.previous = self.current;
        self.current = Some(candidate_id);
        true
    }
    /// Rollback to previous layout if available.
    pub fn rollback(&mut self) -> bool {
        if let Some(prev) = self.previous {
            self.current = Some(prev);
            self.previous = None;
            true
        } else { false }
    }
    pub fn retire_unreferenced(&mut self, referenced: &[u128]) {
        self.layouts.retain(|l| referenced.contains(&l.layout_id) || Some(l.layout_id) == self.current || Some(l.layout_id) == self.previous);
    }
}
