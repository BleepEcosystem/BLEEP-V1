use bleep_crypto::quantum_secure::QuantumSecure;
use crate::transaction::{ZKTransaction, P2PMessage, PeerManager, GossipProtocol, MultiHopRouting, DarkRouting};
use std::sync::Arc;

pub struct TransactionManager {
    peer_manager: Arc<PeerManager>,
    gossip_protocol: Arc<GossipProtocol>,
    multi_hop_routing: Arc<MultiHopRouting>,
    dark_routing: Arc<DarkRouting>,
}

impl TransactionManager {
    pub fn new(
        peer_manager: Arc<PeerManager>,
        gossip_protocol: Arc<GossipProtocol>,
        multi_hop_routing: Arc<MultiHopRouting>,
        dark_routing: Arc<DarkRouting>,
    ) -> Self {
        Self {
            peer_manager,
            gossip_protocol,
            multi_hop_routing,
            dark_routing,
        }
    }

    pub async fn broadcast_transaction(&self, transaction: ZKTransaction) {
        let message = P2PMessage::Transaction(transaction);
        self.gossip_protocol.broadcast_message(message).await;
    }

    pub async fn route_transaction(&self, sender: &str, receiver: &str, transaction: ZKTransaction) {
        let route = self.multi_hop_routing.select_route(sender, receiver).await;
        self.multi_hop_routing.forward_message(route, P2PMessage::Transaction(transaction)).await;
    }

    pub async fn send_anonymous_transaction(&self, sender: &str, transaction: ZKTransaction) {
        let route = self.dark_routing.select_anonymous_route(sender).await;
        self.dark_routing.forward_anonymous(route, P2PMessage::Transaction(transaction)).await;
    }

    pub async fn process_p2p_message(&self, message: P2PMessage) {
        if let P2PMessage::Transaction(tx) = message {
            // Get quantum secure instance
            let quantum_secure = QuantumSecure::keygen();
            if tx.verify(&quantum_secure) {
                self.peer_manager.add_transaction_to_pool(tx);
                log::info!("✅ Valid transaction received and added to mempool.");
            } else {
                log::warn!("❌ Invalid transaction rejected.");
            }
        }
    }
}
