pub mod ai_adaptive_logic;
pub mod blockchain_state;
pub mod consensus;
pub mod networking;
pub mod tests;

pub use consensus::{BLEEPAdaptiveConsensus, ConsensusMode, Validator};
pub use blockchain_state::BlockchainState;
pub use networking::NetworkingModule;

pub fn run_consensus_engine() -> Result<(), Box<dyn std::error::Error>> {
	// TODO: Implement consensus engine
	todo!("run_consensus_engine not yet implemented");
}
