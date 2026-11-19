use ocf_layout::{generate_candidate, LayoutRegistry, WorkloadObserver};

#[test]
fn candidate_is_valid_permutation() {
    let mut obs = WorkloadObserver::new(5);
    obs.record(2); obs.record(2); obs.record(4);
    let cand = generate_candidate(&obs, 1, "hot");
    assert!(cand.is_valid_permutation(5));
    // most accessed chunk (2) should come first
    assert_eq!(cand.chunk_order[0], 2);
}

#[test]
fn publish_and_rollback() {
    let mut reg = LayoutRegistry::new();
    let l1 = ocf_layout::PhysicalLayout::new(1, "seq", (0..4).collect());
    let l2 = ocf_layout::PhysicalLayout::new(2, "hot", vec![3,2,1,0]);
    reg.register(l1); reg.register(l2);
    assert!(reg.publish(1));
    assert_eq!(reg.current, Some(1));
    assert!(reg.publish(2));
    assert_eq!(reg.current, Some(2));
    assert_eq!(reg.previous, Some(1));
    assert!(reg.rollback());
    assert_eq!(reg.current, Some(1));
}

#[test]
fn retire_unreferenced_keeps_current() {
    let mut reg = LayoutRegistry::new();
    let l1 = ocf_layout::PhysicalLayout::new(1, "a", vec![0]);
    let l2 = ocf_layout::PhysicalLayout::new(2, "b", vec![0]);
    reg.register(l1); reg.register(l2);
    reg.publish(2);
    reg.retire_unreferenced(&[2]);
    assert!(reg.get(2).is_some());
    assert!(reg.get(1).is_none());
}
