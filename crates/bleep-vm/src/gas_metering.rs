use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use dashmap::DashMap;
use metrics::{counter, gauge, histogram};
use tracing::{info, warn};

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub enum OpCode {
    Add = 0x01,
    Sub = 0x02,
    Mul = 0x03,
    Div = 0x04,
    Load = 0x05,
    Store = 0x06,
    Call = 0x07,
    Return = 0x08,
}

#[derive(Debug)]
pub struct GasMeter {
    // Base costs for different operations
    op_costs: DashMap<OpCode, u64>,
    
    // Dynamic pricing factors
    memory_factor: AtomicU64,
    compute_factor: AtomicU64,
    storage_factor: AtomicU64,
    
    // Historical data for adaptive pricing
    historical_costs: DashMap<Vec<u8>, Vec<u64>>,
    
    // Runtime statistics
    total_gas_used: AtomicU64,
    peak_gas_used: AtomicU64,
}

#[derive(Debug)]
pub struct GasConfig {
    pub base_memory_gas: u64,
    pub base_compute_gas: u64,
    pub base_storage_gas: u64,
    pub max_gas_per_op: u64,
    pub min_gas_per_op: u64,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            base_memory_gas: 1,
            base_compute_gas: 1,
            base_storage_gas: 10,
            max_gas_per_op: 1000,
            min_gas_per_op: 1,
        }
    }
}

#[derive(Debug)]
pub struct GasReport {
    pub total_gas: u64,
    pub computation_gas: u64,
    pub memory_gas: u64,
    pub storage_gas: u64,
    pub operation_breakdown: HashMap<OpCode, u64>,
}

impl GasMeter {
    pub fn new() -> Self {
        let op_costs = DashMap::new();
        
        // Initialize base operation costs
        op_costs.insert(OpCode::Add, 1);
        op_costs.insert(OpCode::Sub, 1);
        op_costs.insert(OpCode::Mul, 2);
        op_costs.insert(OpCode::Div, 3);
        op_costs.insert(OpCode::Load, 3);
        op_costs.insert(OpCode::Store, 5);
        op_costs.insert(OpCode::Call, 8);
        op_costs.insert(OpCode::Return, 1);

        Self {
            op_costs,
            memory_factor: AtomicU64::new(1),
            compute_factor: AtomicU64::new(1),
            storage_factor: AtomicU64::new(1),
            historical_costs: DashMap::new(),
            total_gas_used: AtomicU64::new(0),
            peak_gas_used: AtomicU64::new(0),
        }
    }

    pub fn calculate_gas(&self, contract: &Vec<u8>) -> u64 {
        let mut total_gas = 0u64;
        let mut operation_gas = HashMap::new();

        // Calculate operation costs
        for chunk in contract.chunks(4) {
            if let Some(op) = self.decode_opcode(chunk) {
                let base_cost = self.op_costs.get(&op).map_or(1, |v| *v);
                let adjusted_cost = self.adjust_cost(base_cost, &operation_gas);
                total_gas += adjusted_cost;
                *operation_gas.entry(op).or_insert(0) += adjusted_cost;
            }
        }

        // Calculate memory costs
        let memory_gas = self.calculate_memory_gas(contract.len());
        total_gas += memory_gas;

        // Calculate storage costs
        let storage_gas = self.calculate_storage_gas(&operation_gas);
        total_gas += storage_gas;

        // Update statistics
        self.update_stats(total_gas);

        // Update historical data
        self.update_historical_data(contract, total_gas);

        total_gas
    }

    pub fn calculate_gas_detailed(&self, contract: &Vec<u8>) -> GasReport {
        let mut operation_breakdown = HashMap::new();
        let mut computation_gas = 0u64;
        
        // Calculate operation-specific gas
        for chunk in contract.chunks(4) {
            if let Some(op) = self.decode_opcode(chunk) {
                let base_cost = self.op_costs.get(&op).map_or(1, |v| *v);
                let adjusted_cost = self.adjust_cost(base_cost, &operation_breakdown);
                computation_gas += adjusted_cost;
                *operation_breakdown.entry(op).or_insert(0) += adjusted_cost;
            }
        }

        // Calculate memory and storage costs
        let memory_gas = self.calculate_memory_gas(contract.len());
        let storage_gas = self.calculate_storage_gas(&operation_breakdown);
        let total_gas = computation_gas + memory_gas + storage_gas;

        GasReport {
            total_gas,
            computation_gas,
            memory_gas,
            storage_gas,
            operation_breakdown,
        }
    }

    pub fn update_pricing_factors(&self, network_load: f64, block_time: u64) {
        // Adjust computation factor based on network load
        let new_compute_factor = ((network_load * 2.0) as u64).max(1);
        self.compute_factor.store(new_compute_factor, Ordering::Relaxed);

        // Adjust storage factor based on block time
        let new_storage_factor = (block_time / 5).max(1);
        self.storage_factor.store(new_storage_factor, Ordering::Relaxed);

        // Log adjustments
        info!(
            "Updated pricing factors - compute: {}, storage: {}", 
            new_compute_factor, 
            new_storage_factor
        );
    }

    // Private helper methods
    fn decode_opcode(&self, bytes: &[u8]) -> Option<OpCode> {
        if bytes.is_empty() {
            return None;
        }
        
        match bytes[0] {
            0x01 => Some(OpCode::Add),
            0x02 => Some(OpCode::Sub),
            0x03 => Some(OpCode::Mul),
            0x04 => Some(OpCode::Div),
            0x05 => Some(OpCode::Load),
            0x06 => Some(OpCode::Store),
            0x07 => Some(OpCode::Call),
            0x08 => Some(OpCode::Return),
            _ => None,
        }
    }

    fn adjust_cost(&self, base_cost: u64, operation_counts: &HashMap<OpCode, u64>) -> u64 {
        let compute_factor = self.compute_factor.load(Ordering::Relaxed);
        let repetition_penalty = operation_counts
            .values()
            .map(|&count| count / 100)
            .sum::<u64>()
            .max(1);

        base_cost * compute_factor * repetition_penalty
    }

    fn calculate_memory_gas(&self, size: usize) -> u64 {
        let memory_factor = self.memory_factor.load(Ordering::Relaxed);
        let size_u64 = size as u64;
        
        // Quadratic scaling for large memory usage
        if size_u64 > 1024 {
            memory_factor * (size_u64 + (size_u64 * size_u64 / 1024))
        } else {
            memory_factor * size_u64
        }
    }

    fn calculate_storage_gas(&self, operations: &HashMap<OpCode, u64>) -> u64 {
    let storage_factor = self.storage_factor.load(Ordering::Relaxed);
    let storage_ops = operations.get(&OpCode::Store).copied().unwrap_or(0);
    storage_factor * storage_ops
    }

    fn update_stats(&self, gas_used: u64) {
        self.total_gas_used.fetch_add(gas_used, Ordering::Relaxed);
        
        let current_peak = self.peak_gas_used.load(Ordering::Relaxed);
        if gas_used > current_peak {
            self.peak_gas_used.store(gas_used, Ordering::Relaxed);
        }

        // Update metrics
    counter!("gas.total", gas_used as u64);
    gauge!("gas.current", gas_used as f64);
    histogram!("gas.distribution", gas_used as f64);
    }

    fn update_historical_data(&self, contract: &Vec<u8>, gas_used: u64) {
        if let Some(mut history) = self.historical_costs.get_mut(contract) {
            history.push(gas_used);
            if history.len() > 100 {
                history.remove(0);
            }
        } else {
            self.historical_costs.insert(contract.clone(), vec![gas_used]);
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_gas_calculation() {
        let meter = GasMeter::new();
        let contract = vec![0x01, 0x02, 0x03, 0x04];
        let gas = meter.calculate_gas(&contract);
        assert!(gas > 0);
    }

    #[test]
    fn test_detailed_gas_report() {
        let meter = GasMeter::new();
        let contract = vec![0x01, 0x02, 0x03, 0x04];
        let report = meter.calculate_gas_detailed(&contract);
        assert!(report.total_gas > 0);
        assert!(report.computation_gas > 0);
    }

    #[test]
    fn test_pricing_factors() {
        let meter = GasMeter::new();
        meter.update_pricing_factors(1.5, 10);
        let contract = vec![0x01, 0x02, 0x03, 0x04];
        let gas1 = meter.calculate_gas(&contract);
        meter.update_pricing_factors(2.0, 20);
        let gas2 = meter.calculate_gas(&contract);
        assert!(gas2 > gas1);
    }
}