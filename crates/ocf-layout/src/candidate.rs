use crate::layout::PhysicalLayout;
use crate::observer::WorkloadObserver;

/// Generate a candidate layout by sorting chunks by access frequency (descending).
pub fn generate_candidate(observer: &WorkloadObserver, layout_id: u128, name: &str) -> PhysicalLayout {
    let mut indices: Vec<usize> = (0..observer.access_counts.len()).collect();
    indices.sort_by(|&a, &b| observer.access_counts[b].cmp(&observer.access_counts[a]));
    PhysicalLayout::new(layout_id, name, indices)
}
