pub struct WorkloadObserver {
    pub access_counts: Vec<u64>,
}

impl WorkloadObserver {
    pub fn new(n_chunks: usize) -> Self { Self { access_counts: vec![0; n_chunks] } }
    pub fn record(&mut self, chunk_idx: usize) {
        if chunk_idx < self.access_counts.len() {
            self.access_counts[chunk_idx] += 1;
        }
    }
    pub fn total_accesses(&self) -> u64 { self.access_counts.iter().sum() }
}
