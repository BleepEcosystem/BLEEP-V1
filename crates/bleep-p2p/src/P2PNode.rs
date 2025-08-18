use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

pub struct PeerManager;
pub struct GossipProtocol;
pub struct BlockchainState;
// P2PMessage defined below
#[derive(Clone)]
pub struct Block;
#[derive(Clone)]
pub struct Transaction;
#[derive(Clone)]
pub enum P2PMessage {
    NewBlock(Block),
    NewTransaction(Transaction),
    ShardState(String),
}

impl P2PMessage {
    pub fn validate(&self) -> Option<String> { Some(String::new()) }
}

impl BlockchainState {
    pub fn add_block(&mut self, _block: Block) -> Result<(), ()> { Ok(()) }
    pub fn add_transaction(&mut self, _transaction: Transaction) -> Result<(), ()> { Ok(()) }
}

impl GossipProtocol {
    pub fn new() -> Self { Self }
    pub fn is_known(&self, _msg: &str) -> bool { false }
    pub fn gossip_message(&self, _node: &P2PNode, _msg: P2PMessage) {}
}

impl PeerManager {
    pub fn new() -> Self { Self }
}

pub struct P2PNode {
    id: String,
    addr: SocketAddr,
    peer_manager: PeerManager,
    gossip_protocol: GossipProtocol,
    blockchain: Arc<Mutex<BlockchainState>>,
}

impl Clone for P2PNode {
    fn clone(&self) -> Self {
        P2PNode {
            id: self.id.clone(),
            addr: self.addr,
            peer_manager: PeerManager::new(),
            gossip_protocol: GossipProtocol::new(),
            blockchain: self.blockchain.clone(),
        }
    }
}

impl P2PNode {
    pub fn broadcast(&self, _msg: P2PMessage) -> Result<(), ()> { Ok(()) }
}

impl P2PNode {
    pub fn new(id: String, addr: SocketAddr, blockchain: Arc<Mutex<BlockchainState>>) -> Self {
        P2PNode {
            id,
            addr,
            peer_manager: PeerManager::new(),
            gossip_protocol: GossipProtocol::new(),
            blockchain,
        }
    }

    pub fn handle_message(&self, message: P2PMessage, peer_addr: SocketAddr) {
        if self.gossip_protocol.is_known(&message.validate().unwrap_or_default()) {
            return;
        }

        match message {
            P2PMessage::NewBlock(block) => {
                let mut blockchain = self.blockchain.lock().unwrap();
                if blockchain.add_block(block).is_ok() {
                    self.gossip_protocol.gossip_message(self, P2PMessage::NewBlock(block));
                }
            }
            P2PMessage::NewTransaction(transaction) => {
                self.blockchain.lock().unwrap().add_transaction(transaction.clone());
                self.gossip_protocol.gossip_message(self, P2PMessage::NewTransaction(transaction));
            }
            _ => {}
        }
    }
}