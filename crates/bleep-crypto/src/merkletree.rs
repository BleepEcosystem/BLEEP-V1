// Stub for MerkleTree
#[derive(Default)]
pub struct MerkleTree;
impl MerkleTree {
    pub fn new() -> Self { MerkleTree }
    pub fn add_leaf(&mut self, _leaf: Vec<u8>) {}
    pub fn contains_leaf(&self, _leaf: &[u8]) -> bool { false }
}
