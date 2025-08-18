use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;
use kademlia_dht::{Kademlia, NodeId};
use ai_security::{PeerScoring, SybilDetector};
use quantum_crypto::{Kyber, SphincsPlus};
use crate::message_protocol::{MessageProtocol, SecureMessage, MessageType};

/// Gossip propagation interval (adjustable)
const GOSSIP_INTERVAL: Duration = Duration::from_secs(3);

/// Secure, AI-enhanced Gossip Protocol for BLEEP
#[derive(Debug)]
pub struct GossipProtocol {
    peers: Arc<Mutex<HashSet<NodeId>>>,  
    seen_messages: Arc<Mutex<HashMap<String, Instant>>>,  
    message_protocol: MessageProtocol,
    peer_scoring: PeerScoring,
    sybil_detector: SybilDetector,
}

impl GossipProtocol {
    /// Initializes the Gossip Protocol with AI-powered peer scoring and Sybil resistance
    pub fn new(message_protocol: MessageProtocol) -> Self {
        Self {
            peers: Arc::new(Mutex::new(HashSet::new())),
            seen_messages: Arc::new(Mutex::new(HashMap::new())),
            message_protocol,
            peer_scoring: PeerScoring::new(),
            sybil_detector: SybilDetector::new(),
        }
    }

    /// Adds a new peer to the gossip network
    pub fn add_peer(&self, peer_id: NodeId) {
        let mut peers = self.peers.lock().unwrap();
        if !self.sybil_detector.is_suspicious(&peer_id) {
            peers.insert(peer_id);
        }
    }

    /// Removes a peer (due to inactivity, malicious behavior, or Sybil attack detection)
    pub fn remove_peer(&self, peer_id: &NodeId) {
        self.peers.lock().unwrap().remove(peer_id);
    }

    /// Checks if a message has already been seen to prevent redundant propagation
    fn is_duplicate(&self, message_id: &str) -> bool {
        let mut seen_messages = self.seen_messages.lock().unwrap();
        if seen_messages.contains_key(message_id) {
            return true;
        }
        seen_messages.insert(message_id.to_string(), Instant::now());
        false
    }

    /// Encrypts a message using quantum-safe Kyber encryption
    fn encrypt_message(&self, message: &SecureMessage) -> Vec<u8> {
        let encrypted_payload = Kyber::encrypt(&message.payload);
        SphincsPlus::sign(&encrypted_payload)
    }

    /// Securely gossips a message to high-scoring peers
    pub async fn gossip_message(&self, message: SecureMessage) {
        let peers = self.peers.lock().unwrap().clone();
        let high_score_peers = self.peer_scoring.top_peers(peers.len() / 2);  

        for peer_id in high_score_peers {
            if let Some(peer_addr) = Kademlia::get_peer_address(&peer_id) {
                let encrypted_payload = self.encrypt_message(&message);
                let secure_message = SecureMessage {
                    sender_id: message.sender_id.clone(),
                    message_type: message.message_type.clone(),
                    payload: encrypted_payload,
                    signature: message.signature.clone(),
                };

                self.message_protocol.send_message(peer_addr, secure_message).await;
            }
        }
    }

    /// Periodic gossip loop for propagating messages efficiently
    pub async fn start_gossip_loop(&self, mut receiver: mpsc::Receiver<SecureMessage>) {
        loop {
            if let Some(message) = receiver.recv().await {
                if !self.is_duplicate(&message.sender_id) {
                    self.gossip_message(message).await;
                }
            }
            sleep(GOSSIP_INTERVAL).await;
        }
    }
}

pub struct MessageProtocol;