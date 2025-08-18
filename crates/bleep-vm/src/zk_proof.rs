

#[derive(Debug, Default, Clone)]
pub struct ZkProof {
    pub proof_data: Vec<u8>,
    pub public_inputs: Vec<u8>,
}

impl ZkProof {
    pub fn to_bytes(&self) -> Vec<u8> {
        [self.proof_data.as_slice(), self.public_inputs.as_slice()].concat()
    }
}
