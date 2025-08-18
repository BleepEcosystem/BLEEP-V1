#[derive(Debug, Default, Clone)]
pub struct ExecutionResult {
    pub output: Option<Vec<u8>>,
    pub gas_used: u64,
    pub execution_metrics: Option<String>,
}
