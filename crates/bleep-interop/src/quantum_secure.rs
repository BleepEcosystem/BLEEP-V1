pub struct QuantumSecure;
impl QuantumSecure {
    pub fn encrypt(&self, _input: &str) -> Result<Vec<u8>, crate::bleep_connect::BLEEPConnectError> {
        Ok(vec![])
    }
}
