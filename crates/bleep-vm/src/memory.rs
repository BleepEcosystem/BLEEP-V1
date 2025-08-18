use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MemoryChunk {
    pub id: usize,
    pub size: usize,
}

impl MemoryChunk {
    pub fn new(size: usize) -> Result<Self, super::errors::VMError> {
        Ok(Self { id: 0, size })
    }
}

#[derive(Debug, Default)]
pub struct MemoryManager;

impl MemoryManager {
    pub fn new(_limit: MemoryLimit) -> Self { Self }
    pub fn peak_usage(&self) -> usize { 0 }
}

#[derive(Debug, Default)]
pub struct MemoryLimit;
