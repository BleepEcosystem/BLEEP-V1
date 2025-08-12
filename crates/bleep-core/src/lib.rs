// === Core Blockchain Logic ===
pub mod block;
pub mod block_validation;
pub mod blockchain;
pub mod state;
pub mod networking;

// === Transactions and Mempool ===
pub mod transaction;
pub mod transaction_manager;
pub mod transaction_pool;
pub mod mempool;

// === Identity and Security ===
pub mod proof_of_identity;
pub mod anti_asset_loss;

// === Re-exports for broader ecosystem access ===
pub use block::{Block};
pub use block_validation::*;
pub use blockchain::*;
pub use transaction::{ZKTransaction};
pub use transaction_manager::*;
pub use transaction_pool::*;
pub use mempool::*;
pub use proof_of_identity::*;
pub use anti_asset_loss::*;

// === Internal Unit Tests ===
#[cfg(test)]
mod tests;
