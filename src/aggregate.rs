use crate::{input::SigScriptRecord, template::IssuedCell};
use ckb_types::{core::Capacity, H160};
use std::collections::BTreeMap;

const SIG_CODE_HASH: &str = "0x";

pub fn reduce_sig_record(records: Vec<SigScriptRecord>) -> Vec<IssuedCell> {
    let mut record_map: BTreeMap<H160, Capacity> = BTreeMap::new();
    for record in records {
        let SigScriptRecord { args, capacity } = record;
        let entry = record_map
            .entry(args.clone())
            .or_insert_with(Capacity::zero);
        if let Err(e) = entry.safe_add(capacity) {
            eprintln!(
                "Warn: record capacity reduce overflow {} {}: {}",
                args, capacity, e
            );
        };
    }
    record_map
        .into_iter()
        .map(|(args, capacity)| IssuedCell {
            capacity: capacity.as_u64(),
            code_hash: SIG_CODE_HASH.to_string(),
            args: format!("0x{:x}", args),
            hash_type: "type".to_string(),
        })
        .collect()
}
