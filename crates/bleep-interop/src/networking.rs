pub struct BLEEPNetworking;

impl BLEEPNetworking {
	pub fn send_filecoin_transaction<A, B, C: ?Sized>(&self, _client: &A, _request: &B, _proof: &C) -> Result<String, crate::bleep_connect::BLEEPConnectError> {
		Ok("dummy_tx_hash".to_string())
	}
	pub fn send_near_transaction<A, B, C: ?Sized>(&self, _client: &A, _request: &B, _proof: &C) -> Result<String, crate::bleep_connect::BLEEPConnectError> {
		Ok("dummy_tx_hash".to_string())
	}
	pub fn send_zksync_transaction<A, B, C: ?Sized>(&self, _client: &A, _request: &B, _proof: &C) -> Result<String, crate::bleep_connect::BLEEPConnectError> {
		Ok("dummy_tx_hash".to_string())
	}
	pub fn send_starknet_transaction<A, B, C: ?Sized>(&self, _client: &A, _request: &B, _proof: &C) -> Result<String, crate::bleep_connect::BLEEPConnectError> {
		Ok("dummy_tx_hash".to_string())
	}
}
