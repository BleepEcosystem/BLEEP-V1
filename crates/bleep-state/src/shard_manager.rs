use crate::state_storage::BlockchainState;
impl ShardManager {
    pub fn get_state_for_shard(&self, _shard_id: u64) -> Option<BlockchainState> {
        // Stub: always return None
        None
    }
}
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::{info, warn, error};
use crate::sharding::{BLEEPShardingModule, BLEEPError};
use crate::p2p::P2PNode;
use crate::transaction::Transaction;
use crate::consensus::BLEEPAdaptiveConsensus;
use crate::p2p::P2PMessage;

// Use Transaction from crate::transaction
// Use BLEEPAdaptiveConsensus from crate::consensus
// Use P2PNode from sharding.rs
// Use P2PMessage from bleep-p2p

// Use BLEEPError from crate::sharding

pub struct Shard {
    pub transactions: std::collections::VecDeque<Transaction>,
    pub load: usize,
}

// Use BLEEPShardingModule from crate::sharding

// Use BLEEPShardingModule implementation from sharding.rs

pub struct ShardManager {
    pub sharding_module: Arc<Mutex<BLEEPShardingModule>>,
}

impl ShardManager {
    /// Initializes the ShardManager with a given number of shards
    pub fn new(
        num_shards: u64,
        consensus: Arc<Mutex<BLEEPAdaptiveConsensus>>,
        p2p_node: Arc<P2PNode>,
    ) -> Result<Self, BLEEPError> {
        let sharding_module = BLEEPShardingModule::new(num_shards, consensus, p2p_node)?;
        Ok(Self {
            sharding_module: Arc::new(Mutex::new(sharding_module)),
        })
    }

    /// Adds a transaction to the appropriate shard
    pub fn add_transaction(&self, transaction: Transaction) -> Result<(), BLEEPError> {
        let mut sharding_module = self.sharding_module.lock().unwrap();
        info!(
            "[ShardManager] Adding transaction ID {} from {} to {} with amount {}",
            transaction.id, transaction.from, transaction.to, transaction.amount
        );

        match sharding_module.assign_transaction(transaction) {
            Ok(_) => Ok(()),
            Err(err) => {
                error!("[ShardManager] Transaction assignment failed: {:?}", err);
                Err(err)
            }
        }
    }

    /// Manually assigns a transaction to a specific shard
    pub fn assign_transaction_to_shard(&self, transaction: Transaction, shard_id: u64) -> Result<(), BLEEPError> {
        let mut sharding_module = self.sharding_module.lock().unwrap();
        
        if let Some(shard) = sharding_module.shards.get(&shard_id) {
            let mut shard_guard = shard.lock().unwrap();
            shard_guard.transactions.push_back(transaction.clone());
            shard_guard.load += 1;
            info!("[ShardManager] Manually assigned transaction ID {} to Shard {}", transaction.id, shard_id);
            sharding_module.persist_shard_state(shard_id);
            Ok(())
        } else {
            error!("[ShardManager] Invalid shard ID: {}", shard_id);
            Err(BLEEPError::InvalidShard)
        }
    }

    /// Triggers a manual rebalance across all shards
    pub fn rebalance_shards(&self) {
        let mut sharding_module = self.sharding_module.lock().unwrap();
        info!("[ShardManager] Triggering manual rebalance across all shards...");
        sharding_module.monitor_and_auto_rebalance();
    }

    /// Broadcasts all shard states to the P2P network for consistency
    pub fn broadcast_shard_states(&self) {
        let sharding_module = self.sharding_module.lock().unwrap();
        info!("[ShardManager] Broadcasting shard states to the P2P network...");
        match sharding_module.p2p_node.broadcast(P2PMessage::ShardState(sharding_module.get_shard_summary())) {
            Ok(_) => info!("[ShardManager] Shard states broadcasted successfully."),
            Err(err) => error!("[ShardManager] Failed to broadcast shard states: {:?}", err),
        }
    }

    /// Returns the current state of all shards
    pub fn get_shard_states(&self) -> HashMap<u64, usize> {
        let sharding_module = self.sharding_module.lock().unwrap();
        sharding_module.shards.iter().map(|(&id, shard)| {
            let shard_guard = shard.lock().unwrap();
            (id, shard_guard.load)
        }).collect()
    }

    /// Checks node health, removing inactive peers
    pub fn check_node_health(&self) {
        let mut p2p_node = self.sharding_module.lock().unwrap().p2p_node.clone();
    let peers = p2p_node.peers().clone();
        
        for peer in peers.iter() {
            if !p2p_node.is_peer_active(peer) {
                warn!("[ShardManager] Peer {:?} is unresponsive; removing from network.", peer);
                p2p_node.remove_peer(peer);
            }
        }
    }
}