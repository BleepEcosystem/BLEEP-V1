use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use log::{info, warn};
use ring::digest;
use pqcrypto_sphincsplus::sphincsshake256fsimple::*;
use pqcrypto_traits::sign::DetachedSignature;
use bincode;

// SPHINCS+ signature size constant
const CRYPTO_SIGN_BYTES: usize = 49856;

use bleep_core::block::Block;
use bleep_core::blockchain::Blockchain;
use crate::blockchain_state::BlockchainState;
use crate::networking::NetworkingModule;
use bleep_crypto::zkp_verification::BLEEPError;
use crate::ai_adaptive_logic::AIAdaptiveConsensus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusMode {
    PoS,   // Proof of Stake
    PBFT,  // Practical Byzantine Fault Tolerance
    PoW,   // Proof of Work
}

#[derive(Debug, Clone)]
pub struct Validator {
    pub id: String,
    pub reputation: f64,
    pub latency: u64,
    pub stake: u64,
    pub active: bool,
    pub last_signed_block: u64,
}

pub struct BLEEPAdaptiveConsensus {
    consensus_mode: ConsensusMode,
    network_reliability: f64,
    validators: HashMap<String, Validator>,
    pow_difficulty: usize,
    networking: Arc<NetworkingModule>,
    ai_engine: Arc<AIAdaptiveConsensus>,
    blockchain: Arc<RwLock<Blockchain>>,
}

impl BLEEPAdaptiveConsensus {
    pub fn new(
        validators: HashMap<String, Validator>,
        networking: Arc<NetworkingModule>,
        ai_engine: Arc<AIAdaptiveConsensus>,
    ) -> Self {
        use bleep_core::transaction_pool::TransactionPool;
        use bleep_core::blockchain::BlockchainState;

        // 1. Create an empty transaction pool (max size 10_000 for example)
    let tx_pool = TransactionPool::new(10_000);

        // 2. Create an initial blockchain state (empty balances)
        let state = BlockchainState::default();

        // 3. Create a genesis block (index 0, no transactions, previous_hash = "0")
        let genesis_block = Block::new(0, vec![], "0".to_string());

        // 4. Initialize the blockchain with these real objects
        let blockchain = Arc::new(RwLock::new(
            Blockchain::new(genesis_block, state, tx_pool)
        ));

        let initial_mode = ConsensusMode::PoS;
        BLEEPAdaptiveConsensus {
            consensus_mode: initial_mode,
            network_reliability: 0.95,
            validators,
            pow_difficulty: 4,
            networking,
            ai_engine,
            blockchain,
        }
    }

    pub fn switch_consensus_mode(&mut self, network_load: u64, avg_latency: u64) {
        let predicted_mode = self.ai_engine.predict_best_consensus();
        if self.consensus_mode != predicted_mode {
            info!("Switching consensus mode to {:?}", predicted_mode);
            self.consensus_mode = predicted_mode;
        }
    }

    pub fn finalize_block(&mut self, block: &Block, state: &mut BlockchainState) -> Result<(), BLEEPError> {
        let success = match self.consensus_mode {
            ConsensusMode::PoS => self.pos_algorithm(block, state),
            ConsensusMode::PBFT => self.pbft_algorithm(block, state),
            ConsensusMode::PoW => self.pow_algorithm(block),
        };

        if success {
            info!("Block finalized successfully using {:?}", self.consensus_mode);
            Ok(())
        } else {
            warn!("Block finalization failed. Adjusting strategy...");
            self.switch_consensus_mode(50, 40);
            self.finalize_block(block, state)
        }
    }

    fn pos_algorithm(&self, block: &Block, state: &mut BlockchainState) -> bool {
        let mut validators_sorted: Vec<&Validator> = self.validators.values().collect();
        validators_sorted.sort_by(|a, b| b.stake.cmp(&a.stake));
        let selected_validator = validators_sorted.first();

        if let Some(validator) = selected_validator {
            if validator.reputation > 0.8 {
                return state.add_block(block.clone()).is_ok();
            }
        }
        false
    }

    fn pow_algorithm(&mut self, block: &Block) -> bool {
        let target = "0".repeat(self.pow_difficulty);
        let mut nonce = 0;
        let mut hasher = digest::Context::new(&digest::SHA256);

        loop {
            hasher.update(format!("{:?}{}", block, nonce).as_bytes());
            let hash = hex::encode(hasher.clone().finish());

            if hash.starts_with(&target) {
                info!("PoW successful: Nonce = {}, Hash = {}", nonce, hash);
                self.adjust_pow_difficulty();
                return true;
            }

            nonce += 1;
            if nonce > 10_000_000 {
                warn!("PoW failed: Max attempts exceeded.");
                return false;
            }
        }
    }

    fn adjust_pow_difficulty(&mut self) {
        let avg_network_hashrate = self.networking.get_network_hashrate();
        if avg_network_hashrate > 500 {
            self.pow_difficulty += 1;
        } else if self.pow_difficulty > 2 {
            self.pow_difficulty -= 1;
        }
        info!("Adjusted PoW difficulty: {}", self.pow_difficulty);
    }

    fn pbft_algorithm(&self, block: &Block, state: &mut BlockchainState) -> bool {
        let leader = self.select_pbft_leader();
        if leader.is_none() {
            return false;
        }
        let leader_id = leader.unwrap().id.clone();

        if !self.networking.broadcast_proposal(&block, &leader_id) {
            return false;
        }

        let prepare_votes = self.collect_votes(block, "prepare");
        if !self.has_quorum(&prepare_votes) {
            warn!("PBFT: Insufficient quorum in prepare phase.");
            return false;
        }

        let commit_votes = self.collect_votes(block, "commit");
        if self.has_quorum(&commit_votes) {
            return state.add_block(block.clone()).is_ok();
        }

        warn!("PBFT: Commit phase failed.");
        false
    }

    fn select_pbft_leader(&self) -> Option<&Validator> {
        let active_validators: Vec<&Validator> = self
            .validators
            .values()
            .filter(|v| v.active && v.reputation > 0.7)
            .collect();

        if active_validators.is_empty() {
            warn!("No eligible PBFT leaders available.");
            return None;
        }

        let leader = active_validators.iter().max_by(|a, b| a.stake.cmp(&b.stake));
        leader.cloned()
    }

    fn collect_votes(&self, block: &Block, phase: &str) -> HashSet<String> {
        info!("Collecting {:?} votes for block {:?}", phase, block);
        let mut votes = HashSet::new();
        for (id, validator) in &self.validators {
            if validator.reputation > 0.75 {
                votes.insert(id.clone());
            }
        }
        votes
    }

    fn has_quorum(&self, votes: &HashSet<String>) -> bool {
        let required_votes = (self.validators.len() as f64 * 0.66).ceil() as usize;
        votes.len() >= required_votes
    }

    pub fn monitor_validators(&mut self) {
        // Detect malicious validators
        let malicious: Vec<_> = self.validators.iter()
            .filter(|(_, v)| v.reputation > 0.8)
            .map(|(k, _)| k.clone())
            .collect();
        
        // Apply penalties to malicious validators
        for id in malicious {
            warn!("Validator {} detected as malicious! Reducing reputation.", id);
            if let Some(validator) = self.validators.get_mut(&id) {
                validator.reputation *= 0.5;
                validator.active = false;
            }
        }
    }

    pub fn sign_block(&self, block: &Block, _validator_id: &str) -> Vec<u8> {
        let (_pk, sk) = keypair();
        let block_bytes = bincode::serialize(&block).unwrap_or_default();
        let block_hash = digest::digest(&digest::SHA256, &block_bytes);
        let signature = detached_sign(block_hash.as_ref(), &sk);
        signature.as_bytes().to_vec()
    }

    pub fn verify_signature(&self, block: &Block, signature: &[u8], validator_id: &str) -> bool {
        if let Some(_) = self.validators.get(validator_id) {
            let (pk, _) = keypair();
            let block_bytes = bincode::serialize(&block).unwrap_or_default();
            let block_hash = digest::digest(&digest::SHA256, &block_bytes);
            if signature.len() != CRYPTO_SIGN_BYTES {
                return false;
            }
            let mut sig_bytes = [0u8; CRYPTO_SIGN_BYTES];
            sig_bytes.copy_from_slice(signature);
            if let Ok(sig) = pqcrypto_sphincsplus::sphincsshake256fsimple::DetachedSignature::from_bytes(&sig_bytes) {
                verify_detached_signature(&sig, block_hash.as_ref(), &pk).is_ok()
            } else {
                false
            }
        } else {
            false
        }
    }
}
