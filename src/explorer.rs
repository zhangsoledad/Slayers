use crate::rpc::RpcClient;
use ckb_types::{
    bytes::Bytes,
    core::{capacity_bytes, BlockView, Capacity, Ratio},
    packed::CellbaseWitness,
    prelude::*,
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

    pub fn collect(&self, map: &mut BTreeMap<Bytes, Capacity>) -> Result<(), Error> {
        if self.rpc.get_tip_block_number()?.value() < self.target + 11 {
            eprintln!("Lina is not ready yet.");
            exit(0);
        }

        let mut rewards = HashMap::with_capacity(42);
        let mut windows = VecDeque::with_capacity(10);

        let progress_bar = ProgressBar::new(self.target + 10);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:60.cyan/blue} {pos:>7}/{len:7} {msg}")
                .progress_chars("##-"),
        );

        for num in 1..11 {
            progress_bar.inc(1);
            if let Some(block) = self.rpc.get_block_by_number(num.into())? {
                let block: BlockView = block.into();
                windows.push_back(block);
            } else {
                exit(0);
            }
        }

        let mut cursor = 12;
        while cursor <= self.target + 11 {
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
            entry.safe_add(primary)?;
            windows.pop_front();
            cursor += 1;
        }
        rewards.retain(|_, &mut r| r > THRESHOLD);

        let total = rewards
            .iter()
            .map(|(_, capacity)| *capacity)
            .try_fold(Capacity::zero(), Capacity::safe_add)?;

        for (lock, capacity) in rewards {
            let reward = TOTAL_REWARD.safe_mul_ratio(Ratio(capacity.as_u64(), total.as_u64()))?;
            let entry = map
                .entry(lock.args().raw_data())
                .or_insert_with(Capacity::zero);
            *entry = entry.safe_add(reward)?;
        }
        progress_bar.finish();
        Ok(())
    }
}
