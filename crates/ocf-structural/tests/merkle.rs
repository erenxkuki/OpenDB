use ocf_core::content_hash;
use ocf_structural::MerkleTree;

#[test]
fn merkle_root_and_proof() {
    let leaves: Vec<[u8;32]> = (0..7).map(|i| content_hash(&[i as u8])).collect();
    let tree = MerkleTree::from_leaves(leaves.clone());
    let root = tree.root().unwrap();
    for (i, leaf) in leaves.iter().enumerate() {
        let proof = tree.proof(i).unwrap();
        assert!(MerkleTree::verify_proof(leaf, &proof, &root));
    }
    // tampered leaf fails
    let mut bad = leaves[0];
    bad[0] ^= 1;
    let proof = tree.proof(0).unwrap();
    assert!(!MerkleTree::verify_proof(&bad, &proof, &root));
}

#[test]
fn empty_tree() {
    let t = MerkleTree::from_leaves(vec![]);
    assert!(t.root().is_none());
}
