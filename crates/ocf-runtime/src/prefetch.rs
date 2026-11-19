use std::collections::VecDeque;

pub struct Prefetcher {
    queue: VecDeque<usize>,
    max_depth: usize,
}

impl Prefetcher {
    pub fn new(max_depth: usize) -> Self { Self { queue: VecDeque::new(), max_depth } }
    pub fn enqueue(&mut self, chunk_idx: usize) {
        if self.queue.len() >= self.max_depth { return; }
        if !self.queue.contains(&chunk_idx) {
            self.queue.push_back(chunk_idx);
        }
    }
    pub fn dequeue(&mut self) -> Option<usize> { self.queue.pop_front() }
    pub fn len(&self) -> usize { self.queue.len() }
    pub fn is_empty(&self) -> bool { self.queue.is_empty() }
}
