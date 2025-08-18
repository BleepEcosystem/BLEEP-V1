// Stub for P2PNode and P2PMessage for bleep-state
#[derive(Debug, Clone)]
pub struct P2PNode;

#[derive(Debug, Clone)]
pub enum P2PMessage {
    NewBlock(String),
    NewTransaction(String),
    ShardState(String),
}

impl P2PNode {
    pub fn broadcast(&self, _msg: P2PMessage) -> Result<(), ()> { Ok(()) }
    pub fn clone(&self) -> Self { Self }

    // Stub: Returns a list of peer identifiers (as Vec<String>)
    pub fn peers(&self) -> Vec<String> {
        vec!["peer1".to_string(), "peer2".to_string()]
    }

    // Stub: Checks if a peer is active
    pub fn is_peer_active(&self, _peer: &str) -> bool {
        true
    }

    // Stub: Removes a peer from the node
    pub fn remove_peer(&mut self, _peer: &str) {
        // No-op stub
    }
}
