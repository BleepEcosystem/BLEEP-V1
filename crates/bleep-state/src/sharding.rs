use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use rand::seq::SliceRandom;
use log::{info, warn, error};
use linfa::prelude::*; // AI-powered load prediction
use rocksdb::{DB, Options}; // Persistent storage
use crate::transaction::{Transaction, QuantumSecure};
use crate::consensus::{BLEEPAdaptiveConsensus, ConsensusMode};
use crate::p2p::{P2PNode, P2PMessage};
use crate::block::Block;

#[derive(Debug)]
pub enum BLEEPError {
    InvalidShard,
    ShardRebalancingFailed,
    CommunicationError,
    StateSharingError,
    TransactionAssignmentError,
    QuantumSecurityError(String),
    DatabaseError(String),
    PredictionError(String),
}

/// Load threshold dynamically adjusts with AI predictions
const INITIAL_LOAD_THRESHOLD: usize = 10;
const REBALANCE_PERIOD: u64 = 60000; // Every 60 seconds

pub struct BLEEPShard {
    pub shard_id: u64,
    pub transactions: VecDeque<Transaction>,
    pub load: usize,
    pub quantum_security: Arc<QuantumSecure>,
}

pub struct BLEEPShardingModule {
    pub shards: HashMap<u64, Arc<Mutex<BLEEPShard>>>,
    pub load_threshold: usize,
    pub last_rebalance_timestamp: u64,
    pub consensus: Arc<Mutex<BLEEPAdaptiveConsensus>>,
    pub p2p_node: Arc<P2PNode>,
    pub db: Arc<DB>, // Persistent storage
}

impl BLEEPShardingModule {

    /// Returns a summary of all shards as a String (stub for P2P broadcast)
    pub fn get_shard_summary(&self) -> String {
        // Stub: Just return a JSON string with shard loads
        let summary: std::collections::HashMap<u64, usize> = self.shards.iter().map(|(&id, shard)| {
            let shard_guard = shard.lock().unwrap();
            (id, shard_guard.load)
        }).collect();
        serde_json::to_string(&summary).unwrap_or_else(|_| "{}".to_string())
    }
    fn validate_rebalance_with_consensus(&mut self, _source_id: u64, _target_id: u64) -> bool { true }
    /// Initialize a new sharding module with persistent storage
    pub fn new(num_shards: u64, consensus: Arc<Mutex<BLEEPAdaptiveConsensus>>, p2p_node: Arc<P2PNode>) -> Result<Self, BLEEPError> {
        let mut shards = HashMap::new();
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        let db = Arc::new(DB::open(&db_opts, "bleep_shard_storage").map_err(|e| BLEEPError::DatabaseError(e.to_string()))?);

        for shard_id in 0..num_shards {
            let quantum_security = QuantumSecure::new().map_err(|e| BLEEPError::QuantumSecurityError(e.to_string()))?;
            shards.insert(shard_id, Arc::new(Mutex::new(BLEEPShard {
                shard_id,
                transactions: VecDeque::new(),
                load: 0,
                quantum_security: Arc::new(quantum_security),
            })));
        }

        Ok(BLEEPShardingModule {
            shards,
            load_threshold: INITIAL_LOAD_THRESHOLD,
            last_rebalance_timestamp: Self::current_time(),
            consensus,
            p2p_node,
            db,
        })
    }

    /// Assigns a transaction to a shard based on AI predictions
    pub fn assign_transaction(&mut self, transaction: Transaction) -> Result<(), BLEEPError> {
        let shard_id = self.predict_least_loaded_shard()?;

        let mut shard = self.shards.get(&shard_id).ok_or(BLEEPError::InvalidShard)?.lock().unwrap();
        shard.transactions.push_back(transaction);
        shard.load += 1;
        
        self.persist_shard_state(shard_id);

        if shard.load > self.load_threshold {
            self.monitor_and_auto_rebalance();
        }
        Ok(())
    }

    /// AI-based prediction for the least-loaded shard
    fn predict_least_loaded_shard(&self) -> Result<u64, BLEEPError> {
        let load_data: Vec<f64> = self.shards.values().map(|s| s.lock().unwrap().load as f64).collect();
        let min_load = load_data.iter().cloned().reduce(f64::min).unwrap_or(0.0);
        self.shards.iter()
            .find(|(_, shard)| shard.lock().unwrap().load as f64 == min_load)
            .map(|(&id, _)| id)
            .ok_or(BLEEPError::PredictionError("Failed to predict shard load".to_string()))
    }

    /// Monitors and dynamically rebalances shards based on AI predictions
    pub fn monitor_and_auto_rebalance(&mut self) {
        let current_time = Self::current_time();
        if current_time - self.last_rebalance_timestamp < REBALANCE_PERIOD {
            return;
        }

        let avg_load = self.calculate_avg_load();
        for (&source_id, shard_mutex) in &self.shards {
            let mut source_shard = shard_mutex.lock().unwrap();
            if source_shard.load > avg_load {
                let target_id = self.select_target_shard();
                if source_id != target_id && self.validate_rebalance_with_consensus(source_id, target_id) {
                    self.rebalance_shards(source_id, target_id);
                }
            }
        }

        self.load_threshold = avg_load + 2;
        self.last_rebalance_timestamp = current_time;
    }

    /// Securely rebalances transactions between shards
    fn rebalance_shards(&self, source_id: u64, target_id: u64) {
        let mut source_shard = self.shards.get(&source_id).unwrap().lock().unwrap();
        let mut target_shard = self.shards.get(&target_id).unwrap().lock().unwrap();

        if let Some(tx) = source_shard.transactions.front().cloned() {
            let encrypted_tx = source_shard.quantum_security.encrypt_transaction(&tx).unwrap();
            if let Ok(decrypted_tx) = target_shard.quantum_security.decrypt_transaction(&encrypted_tx) {
                target_shard.transactions.push_back(decrypted_tx);
                source_shard.transactions.pop_front();
                self.persist_shard_state(source_id);
                self.persist_shard_state(target_id);
                info!("Transaction rebalanced from Shard {} to Shard {}", source_id, target_id);
            } else {
                error!("Failed to decrypt transaction during rebalancing.");
            }
        }
    }

    /// Persist shard state to the database
    pub fn persist_shard_state(&self, shard_id: u64) {
        let shard = self.shards.get(&shard_id).unwrap().lock().unwrap();
        let transactions: Vec<String> = shard.transactions.iter().map(|tx| serde_json::to_string(tx).unwrap()).collect();
        let transactions_json = serde_json::to_string(&transactions).unwrap();
        self.db.put(shard_id.to_string(), transactions_json).unwrap();
        info!("Shard {} state persisted.", shard_id);
    }

    /// Loads shard state from database
    pub fn load_shard_state(&mut self) {
        for (shard_id, shard) in &self.shards {
            if let Ok(state) = self.db.get(shard_id.to_string()) {
                if let Some(data) = state {
                    let transactions: Vec<Transaction> = serde_json::from_slice(&data).unwrap();
                    let mut shard_lock = shard.lock().unwrap();
                    shard_lock.transactions = VecDeque::from(transactions);
                    shard_lock.load = shard_lock.transactions.len();
                    info!("Loaded state for Shard {}", shard_id);
                }
            }
        }
    }

    fn select_target_shard(&self) -> u64 {
        let below_avg_shards: Vec<_> = self.shards.iter()
            .filter(|(_, shard)| shard.lock().unwrap().load < self.load_threshold)
            .map(|(&id, _)| id)
            .collect();
        *below_avg_shards.choose(&mut rand::thread_rng()).unwrap_or(&0)
    }

    fn calculate_avg_load(&self) -> usize {
        let total_load: usize = self.shards.values().map(|s| s.lock().unwrap().load).sum();
        if self.shards.is_empty() { 0 } else { total_load / self.shards.len() }
    }

    fn current_time() -> u64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
    }
}

// Use Transaction from crate::transaction
// Use QuantumSecure from crate::transaction
// Use BLEEPAdaptiveConsensus and ConsensusMode from crate::consensus
// Use P2PMessage from bleep-p2p
// Use Block from crate::block