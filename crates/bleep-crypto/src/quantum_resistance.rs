use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;
use tokio::sync::RwLock;
use sha3::{Digest, Sha3_256};
use pqcrypto_sphincsplus::sphincsshake256fsimple;
use pqcrypto_traits::sign::{PublicKey as _, DetachedSignature as _};


// Initialize logging
fn init_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
}

// 🔹 Quantum-Resistant Transaction Structure
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Transaction {
    pub id: u64,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl Transaction {
    pub fn new(id: u64, from: &str, to: &str, amount: u64, sk: &sphincsshake256fsimple::SecretKey, pk: &sphincsshake256fsimple::PublicKey) -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut tx = Transaction {
            id,
            from: from.to_string(),
            to: to.to_string(),
            amount,
            timestamp,
            signature: vec![],
            public_key: pk.as_bytes().to_vec(),
        };
        tx.signature = tx.sign(sk);
        tx
    }

    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        hasher.update(&self.id.to_be_bytes());
        hasher.update(self.from.as_bytes());
        hasher.update(self.to.as_bytes());
        hasher.update(&self.amount.to_be_bytes());
        hasher.update(&self.timestamp.to_be_bytes());
        hasher.update(&self.public_key);
        hasher.finalize().to_vec()
    }

    pub fn sign(&self, sk: &sphincsshake256fsimple::SecretKey) -> Vec<u8> {
        let hash = self.hash();
        let sig = sphincsshake256fsimple::detached_sign(&hash, sk);
        sig.as_bytes().to_vec()
    }

    pub fn verify(&self) -> bool {
        let hash = self.hash();
        let pk = match sphincsshake256fsimple::PublicKey::from_bytes(&self.public_key) {
            Ok(pk) => pk,
            Err(_) => return false,
        };
        let sig = match sphincsshake256fsimple::DetachedSignature::from_bytes(&self.signature) {
            Ok(sig) => sig,
            Err(_) => return false,
        };
        sphincsshake256fsimple::verify_detached_signature(&sig, &hash, &pk).is_ok()
    }
}

// 🔹 Blockchain Block Structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub previous_hash: String,
    pub transactions: Vec<Transaction>,
    pub timestamp: u64,
    pub hash: String,
}

impl Block {
    pub fn new(id: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time error")
            .as_secs();
        let hash = Self::calculate_hash(&transactions, &previous_hash, timestamp);
        Block {
            id,
            previous_hash,
            transactions,
            timestamp,
            hash,
        }
    }

    pub fn calculate_hash(transactions: &[Transaction], previous_hash: &str, timestamp: u64) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(previous_hash.as_bytes());
        hasher.update(&timestamp.to_be_bytes());
        for tx in transactions {
            hasher.update(&bincode::serialize(tx).unwrap());
        }
        hex::encode(hasher.finalize())
    }
}

// Quantum-Secure Encryption & Decryption System removed

// 🔹 Blockchain State Management
pub struct BlockchainState {
    pub chain: Arc<RwLock<Vec<Block>>>,
    pub mempool: Arc<RwLock<HashSet<Transaction>>>,
}

impl BlockchainState {
    pub fn new() -> Self {
        BlockchainState {
            chain: Arc::new(RwLock::new(vec![Block::new(0, String::new(), vec![])])),
            mempool: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn add_transaction(&self, transaction: Transaction) {
        let mut mempool = self.mempool.write().await;
        mempool.insert(transaction);
        info!("Transaction added to mempool.");
    }

    pub async fn add_block(&self, block: Block) {
        let mut chain = self.chain.write().await;
        chain.push(block);
        info!("New block added to blockchain.");
    }
}

// 🔹 Adaptive Consensus System
pub struct AdaptiveConsensus {
    pub validators: HashMap<String, f64>,
    pub network_reliability: f64,
    pub consensus_mode: String,
}

impl AdaptiveConsensus {
    pub fn new() -> Self {
        AdaptiveConsensus {
            validators: HashMap::new(),
            network_reliability: 0.9,
            consensus_mode: "PoS".to_string(),
        }
    }

    pub fn switch_mode(&mut self, network_load: u64) {
        self.consensus_mode = if network_load > 80 {
            "PoW".to_string()
        } else if network_load > 40 {
            "PBFT".to_string()
        } else {
            "PoS".to_string()
        };
        info!("Consensus mode switched to {}", self.consensus_mode);
    }
}

// 🔹 Main Blockchain Initialization
#[tokio::main]
async fn main() {
    init_logger();
    let _blockchain = BlockchainState::new();
    let _consensus = AdaptiveConsensus::new();
    info!("Blockchain initialized with genesis block.");
}
