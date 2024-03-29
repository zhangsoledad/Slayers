mod id_generator;
#[macro_use]
mod macros;
mod error;

use ckb_jsonrpc_types::{BlockNumber, BlockReward, BlockView, EpochNumber, EpochView, HeaderView};
use ckb_types::H256;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .gzip(true)
        .timeout(::std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client build");
}

jsonrpc!(pub struct RpcClient {
    pub fn get_block_by_number(&self, _number: BlockNumber) -> Option<BlockView>;
    pub fn get_header_by_number(&self, _number: EpochNumber) -> Option<HeaderView>;
    pub fn get_block_hash(&self, _number: BlockNumber) -> Option<H256>;
    pub fn get_cellbase_output_capacity_details(&self, _hash: H256) -> Option<BlockReward>;
    pub fn get_tip_header(&self) -> HeaderView;
    pub fn get_epoch_by_number(&self, number: EpochNumber) -> Option<EpochView>;
});
