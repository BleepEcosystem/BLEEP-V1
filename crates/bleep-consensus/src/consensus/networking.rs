use bleep_core::block::Block;
use bleep_core::networking::NetworkingModule as CoreNetworkingModule;

pub struct NetworkingModule {
    pub inner: CoreNetworkingModule,
}

impl NetworkingModule {
    pub fn new() -> Self {
        NetworkingModule {
            inner: CoreNetworkingModule::new(),
        }
    }

    pub fn get_network_hashrate(&self) -> u64 {
        // TODO: Implement actual network hashrate calculation
        1000000 // Default value for now
    }

    pub fn broadcast_proposal(&self, block: &Block, _leader_id: &str) -> bool {
        // Forward to broadcast_block for now
        self.inner.broadcast_block(block)
    }

    pub fn broadcast_block(&self, block: &Block) -> bool {
        self.inner.broadcast_block(block)
    }

    pub fn receive_block(&self, block: Block) -> bool {
        self.inner.receive_block(block)
    }
}
