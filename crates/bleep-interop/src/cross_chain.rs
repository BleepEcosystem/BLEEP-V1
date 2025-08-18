#[derive(Default)]
pub struct CrossChainRequest {
	pub from_chain: String,
}

#[derive(Default)]
pub struct CrossChainResponse {
	pub status: String,
	pub transaction_id: String,
	pub confirmation: String,
}
