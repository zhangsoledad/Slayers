use crate::rpc::RpcClient;
use ckb_rational::RationalU256;
use ckb_types::{
    bytes::Bytes,
    core::{capacity_bytes, BlockView, Capacity, HeaderView},
    packed::{Byte32, CellbaseWitness},
    prelude::*,
    U256,
};
use failure::Error;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::process::exit;

const TOTAL_REWARD: Capacity = capacity_bytes!(18_000_000);
const THRESHOLD: Capacity = capacity_bytes!(1_000);

pub struct Explorer {
    rpc: RpcClient,
    target: u64,
}

impl Explorer {
    pub fn new(url: &str, target: u64) -> Explorer {
        Explorer {
            rpc: RpcClient::new(url),
            target,
        }
    }

    pub fn collect(&self, map: &mut BTreeMap<Bytes, Capacity>) -> Result<(u64, Byte32), Error> {
        let tip_header: HeaderView = self.rpc.get_tip_header()?.into();
        let tip_epoch = tip_header.epoch();
        if (tip_epoch.number() < (self.target + 1)) || tip_epoch.index() < 11 {
            eprintln!("Lina is not ready yet.");
            exit(0);
        }

        let netx_epoch = self
            .rpc
            .get_epoch_by_number((self.target + 1).into())?
            .unwrap_or_else(|| exit(0));

        let netx_epoch_start: u64 = netx_epoch.start_number.into();

        let endpoint = netx_epoch_start - 1;
        println!("Explorer endpoint {}", endpoint);

        let mut rewards = HashMap::with_capacity(42);
        let mut windows = VecDeque::with_capacity(10);

        let progress_bar = ProgressBar::new(endpoint + 11);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:60.cyan/blue} {pos:>7}/{len:7} {msg}")
                .progress_chars("##-"),
        );

        for num in 1..=11 {
            progress_bar.inc(1);
            if let Some(block) = self.rpc.get_block_by_number(num.into())? {
                let block: BlockView = block.into();
                windows.push_back(block);
            } else {
                exit(0);
            }
        }

        for cursor in 12..=(endpoint + 11) {
            progress_bar.inc(1);
            if let Some(block) = self.rpc.get_block_by_number(cursor.into())? {
                let block: BlockView = block.into();
                windows.push_back(block);
            } else {
                exit(0);
            }

            let hash = self
                .rpc
                .get_block_hash(cursor.into())?
                .unwrap_or_else(|| exit(0));

            let reward = self
                .rpc
                .get_cellbase_output_capacity_details(hash)?
                .unwrap_or_else(|| exit(0));
            let target_lock = CellbaseWitness::from_slice(
                &windows[0].transactions()[0]
                    .witnesses()
                    .get(0)
                    .expect("target witness exist")
                    .raw_data(),
            )
            .expect("cellbase loaded from store should has non-empty witness")
            .lock();

            let entry = rewards.entry(target_lock).or_insert_with(Capacity::zero);
            let primary: u64 = reward.primary.into();

            *entry = entry.safe_add(primary)?;
            if cursor != endpoint + 11 {
                windows.pop_front();
            }
        }
        let chosen_one = windows.pop_front().unwrap_or_else(|| exit(0));
        rewards.retain(|_, &mut r| r > THRESHOLD);

        let total = rewards
            .iter()
            .map(|(_, capacity)| *capacity)
            .try_fold(Capacity::zero(), Capacity::safe_add)?;

        for (lock, capacity) in rewards {
            let ratio =
                RationalU256::new(U256::from(capacity.as_u64()), U256::from(total.as_u64()));
            let total = RationalU256::new(U256::from(TOTAL_REWARD.as_u64()), U256::one());
            let reward = (total * ratio).into_u256();
            let entry = map
                .entry(lock.args().raw_data())
                .or_insert_with(Capacity::zero);
            *entry = entry.safe_add(get_low64(&reward))?;
        }
        progress_bar.finish();
        Ok((chosen_one.timestamp(), chosen_one.hash()))
    }
}

fn get_low64(u256: &U256) -> u64 {
    u256.0[0]
}
