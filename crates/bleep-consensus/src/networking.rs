use bleep_core::networking::NetworkingModule as CoreNetworkingModule;
use bleep_core::block::Block;

pub struct NetworkingModule {
    inner: CoreNetworkingModule,
}

impl NetworkingModule {
    pub fn new() -> Self {
        Self {
            inner: CoreNetworkingModule::new()
        }
    }

    pub fn get_network_hashrate(&self) -> u64 {
        1000000 // Default value for now
    }

    pub fn broadcast_proposal(&self, block: &Block, leader_id: &str) -> bool {
        self.inner.broadcast_block(block)
    }
}
