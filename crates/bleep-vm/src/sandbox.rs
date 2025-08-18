#[derive(Debug, Default)]
pub struct SecurityPolicy;

impl SecurityPolicy {
    pub fn default() -> Self { Self }
    pub fn validate(&self, _contract: &[u8]) -> Result<(), String> { Ok(()) }
}
