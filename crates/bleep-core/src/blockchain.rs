use std::collections::{VecDeque, HashMap};
use crate::block::Block;
use crate::block_validation::BlockValidator;
use crate::transaction_pool::TransactionPool;
use std::sync::{Arc, RwLock};

/// Represents the current state of the blockchain
#[derive(Default)]
pub struct BlockchainState {
    pub balances: HashMap<String, u64>,
}

impl BlockchainState {
    pub fn revert_block(&mut self, block: &Block) {
        // Revert the changes made by this block
        for tx in &block.transactions {
            // Match the transaction fields from whatever transaction type is used
            match tx {
                crate::block::Transaction { sender, receiver, amount, .. } => {
                    // Now we have proper access to fields
                    let sender = sender.clone();
                    let receiver = receiver.clone();
                    let amount = *amount;
                    
                    // Return amount to sender, remove from receiver
                    *self.balances.entry(sender).or_default() += amount;
                    if let Some(balance) = self.balances.get_mut(&receiver) {
                        *balance = balance.saturating_sub(amount);
                    }
                }
            }
        }
    }
}

pub struct Blockchain {
    pub chain: VecDeque<Block>,
    pub state: Arc<RwLock<BlockchainState>>,
    pub transaction_pool: Arc<RwLock<Arc<TransactionPool>>>,
}

impl Blockchain {
    /// **Initialize blockchain with genesis block**
    pub fn new(genesis_block: Block, state: BlockchainState, tx_pool: Arc<TransactionPool>) -> Self {
        let mut chain = VecDeque::new();
        chain.push_back(genesis_block);

        Self {
            chain,
            state: Arc::new(RwLock::new(state)),
            transaction_pool: Arc::new(RwLock::new(tx_pool)),
        }
    }

    /// **Validate and add a new block to the chain**
    pub fn add_block(&mut self, block: Block, public_key: &[u8]) -> bool {
        let last_block = self.chain.back().unwrap();

        // **Full Validation: Block + Transactions + Network Consensus**
        if !BlockValidator::validate_full_block(last_block, &block, public_key) {
            log::error!("Block {} failed validation.", block.index);
            return false;
        }

        // **Transaction Check: Remove included transactions from pool**
        // Stub: transaction removal

        // **State Update Before Adding Block**
        // Stub: state update

        let block_index = block.index;
        self.chain.push_back(block);
        log::info!("Block {} successfully added to the blockchain.", block_index);

        // **Broadcast to network peers**
        // Stub: broadcast to peers

        true
    }

    /// **Verify the integrity of the entire blockchain**
    pub fn verify_chain(&self, public_key: &[u8]) -> bool {
        for i in 1..self.chain.len() {
            let prev = &self.chain[i - 1];
            let current = &self.chain[i];

            if !BlockValidator::validate_block_link(prev, current)
                || !BlockValidator::validate_block(current, public_key) {
                log::error!("Blockchain integrity check failed at Block {}.", current.index);
                return false;
            }
        }
        log::info!("Blockchain integrity verified successfully.");
        true
    }

    /// **Handle potential chain forks**
    pub fn handle_fork(&mut self, new_chain: VecDeque<Block>) {
        if new_chain.len() > self.chain.len() {
            log::warn!("Fork detected! Switching to longer valid chain.");
            self.chain = new_chain;
        }
    }

    /// **Rollback blockchain state if a block is found invalid later**
    pub fn rollback(&mut self) {
        if let Some(removed_block) = self.chain.pop_back() {
            let mut state = self.state.write().unwrap();
            state.revert_block(&removed_block);
            log::warn!("Block {} removed due to rollback.", removed_block.index);
        }
    }

    /// **Fetch a block by index**
    pub fn get_block_by_index(&self, index: u64) -> Option<&Block> {
        self.chain.iter().find(|b| b.index == index)
    }

    /// **Fetch a block by hash**
    pub fn get_block_by_hash(&self, hash: &str) -> Option<&Block> {
        self.chain.iter().find(|b| b.compute_hash() == hash)
    }
}
