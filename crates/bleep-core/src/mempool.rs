
use crate::transaction::ZKTransaction;


// use crate::core::transaction::ZKTransaction;
// use crate::crypto::proof_of_identity::ProofOfIdentity;
// use crate::networking::encryption::QuantumEncryption;
use std::collections::{HashMap, HashSet};
use tokio::sync::Mutex;
use std::sync::Arc;

/// The Mempool stores unconfirmed transactions before they are added to a block
pub struct Mempool {
    transactions: Mutex<HashMap<String, ZKTransaction>>,  // Stores transactions with unique IDs
    seen_transactions: Mutex<HashSet<String>>,           // Prevents duplicate transactions
}

impl Mempool {
    /// Initializes a new mempool
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            transactions: Mutex::new(HashMap::new()),
            seen_transactions: Mutex::new(HashSet::new()),
        })
    }

    /// Adds a transaction to the mempool after verifying its validity
    pub async fn add_transaction(&self, transaction: ZKTransaction) -> bool {
        let mut transactions = self.transactions.lock().await;
        let mut seen_transactions = self.seen_transactions.lock().await;
        
        let tx_id = transaction.get_hash();
        
        // Check for duplicate transactions
        if seen_transactions.contains(&tx_id) {
            return false;
        }
        
        // Verify transaction signature before adding
        // NOTE: You must pass a QuantumSecure instance to this function in real usage
        // For now, this is a placeholder and will not compile until the function signature is updated
        false
    }

    /// Removes a transaction after it is included in a block
    pub async fn remove_transaction(&self, tx_id: &str) {
        let mut transactions = self.transactions.lock().await;
        let mut seen_transactions = self.seen_transactions.lock().await;
        
        transactions.remove(tx_id);
        seen_transactions.remove(tx_id);
    }

    /// Returns a list of pending transactions for block inclusion
    pub async fn get_pending_transactions(&self) -> Vec<ZKTransaction> {
        let transactions = self.transactions.lock().await;
        transactions.values().cloned().collect()
    }

    /// Checks if a transaction already exists in the mempool
    pub async fn transaction_exists(&self, tx_id: &str) -> bool {
        let transactions = self.transactions.lock().await;
        transactions.contains_key(tx_id)
    }

    /// Clears old transactions (used for mempool cleanup)
    pub async fn clear_old_transactions(&self) {
        let mut transactions = self.transactions.lock().await;
        transactions.retain(|_, tx| tx.timestamp + 600 > chrono::Utc::now().timestamp() as u64);
    }
}

impl ZKTransaction {
    /// Generates a unique hash for the transaction
    pub fn get_hash(&self) -> String {
        format!("{}:{}:{}:{}", self.sender, self.receiver, self.amount, self.timestamp)
    }
}
