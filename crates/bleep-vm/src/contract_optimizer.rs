#[derive(Debug, Default, Clone)]
pub struct ContractOptimizer;

impl ContractOptimizer {
    pub fn new() -> Self { Self }
    pub fn optimize(&self, contract: &[u8], _level: super::optimizer::OptimizationLevel) -> Result<Vec<u8>, super::errors::VMError> {
        Ok(contract.to_vec())
    }
}
