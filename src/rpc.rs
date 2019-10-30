mod id_generator;
#[macro_use]
mod macros;
mod error;

use ckb_jsonrpc_types::{
    Alert, BannedAddr, Block, BlockNumber, BlockReward, BlockTemplate, BlockView, Capacity,
    CellOutputWithOutPoint, CellTransaction, CellWithStatus, ChainInfo, Cycle, DryRunResult,
    EpochNumber, EpochView, HeaderView, LiveCell, LockHashIndexState, Node, OutPoint, PeerState,
    Timestamp, Transaction, TransactionWithStatus, TxPoolInfo, Uint64, Version,
};
use ckb_types::core::{
    BlockNumber as CoreBlockNumber, Capacity as CoreCapacity, EpochNumber as CoreEpochNumber,
    Version as CoreVersion,
};
use ckb_types::{packed::Byte32, prelude::*, H256};
use failure::Error;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .gzip(true)
        .timeout(::std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client build");
}

jsonrpc!(pub struct RpcClient {
    pub fn get_block(&self, _hash: H256) -> Option<BlockView>;
    pub fn get_block_by_number(&self, _number: BlockNumber) -> Option<BlockView>;
    pub fn get_header(&self, _hash: H256) -> Option<HeaderView>;
    pub fn get_header_by_number(&self, _number: BlockNumber) -> Option<HeaderView>;
    pub fn get_transaction(&self, _hash: H256) -> Option<TransactionWithStatus>;
    pub fn get_block_hash(&self, _number: BlockNumber) -> Option<H256>;
    pub fn get_tip_header(&self) -> HeaderView;
    pub fn get_tip_block_number(&self) -> BlockNumber;
    pub fn get_current_epoch(&self) -> EpochView;
    pub fn get_epoch_by_number(&self, number: EpochNumber) -> Option<EpochView>;
    pub fn local_node_info(&self) -> Node;
    pub fn get_blockchain_info(&self) -> ChainInfo;
});
