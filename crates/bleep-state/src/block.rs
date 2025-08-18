// Stub for Block
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub id: u64,
    pub data: String,
}
