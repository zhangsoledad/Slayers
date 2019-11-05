use crate::address::Address;
use ckb_types::{bytes::Bytes, core::Capacity};
use failure::Error;
use serde_derive::Deserialize;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::io::Read;
use std::process::exit;

// pub struct H264([u8; 33]);

const BYTE_SHANNONS: u64 = 100_000_000;

#[derive(Debug, Deserialize)]
pub struct RawRecord {
    pub address: String,
    pub capacity: u64,
}

// #[derive(Debug, Deserialize)]
// pub struct LockRecord {
//     pub address: String,
//     pub capacity: u64,
//     pub date: u64,
// }

pub struct SigScriptRecord {
    pub args: Bytes,
    pub capacity: Capacity,
}

// pub struct MulSigScriptRecord {
//     pub args: Bytes,
//     pub capacity: Capacity,
// }

pub fn parse_mining_competition_record<R: Read>(reader: R, map: &mut BTreeMap<Bytes, Capacity>) {
    let mut rdr = csv::Reader::from_reader(reader);
    let records: Vec<SigScriptRecord> = rdr
        .deserialize()
        .filter_map(|record: Result<RawRecord, _>| record.ok().and_then(|r| r.try_into().ok()))
        .collect();

    for record in records {
        let SigScriptRecord { args, capacity } = record;
        let entry = map.entry(args.clone()).or_insert_with(Capacity::zero);

        *entry = entry.safe_add(capacity).unwrap_or_else(|e| {
            eprintln!("Warn: record capacity reduce overflow: {}", e);
            exit(1);
        });
    }
}

impl TryFrom<RawRecord> for SigScriptRecord {
    type Error = Error;

    fn try_from(record: RawRecord) -> Result<Self, Self::Error> {
        let address = Address::from_str(&record.address)?;
        Ok(SigScriptRecord {
            args: address.args,
            capacity: Capacity::shannons(record.capacity * BYTE_SHANNONS),
        })
    }
}
