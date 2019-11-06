use crate::{
    address::Address,
    date::{parse_date, Outset},
};
use ckb_types::{bytes::Bytes, core::Capacity};
use failure::Error;
use serde_derive::Deserialize;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::io::Read;
use std::process::exit;

const BYTE_SHANNONS: u64 = 100_000_000;

#[derive(Debug, Deserialize)]
pub struct RawRecord {
    pub address: String,
    pub capacity: u64,
}

#[derive(Debug, Deserialize)]
pub struct LockRecord {
    pub address: String,
    pub capacity: u64,
    pub lock: String,
}

pub struct SigScriptRecord {
    pub args: Bytes,
    pub capacity: Capacity,
}

pub struct MulSigScriptRecord {
    pub args: Bytes,
    pub capacity: Capacity,
    pub since: u64,
}

pub fn parse_lock_record<R: Read>(reader: R, map: &mut BTreeMap<Bytes, Capacity>, target: u64) {
    let mut rdr = csv::Reader::from_reader(reader);
    let records: Vec<MulSigScriptRecord> = rdr
        .deserialize()
        .filter_map(|record: Result<LockRecord, _>| {
            record
                .ok()
                .and_then(|r| convert_lock_record(r, target).ok())
        })
        .collect();

    for record in records {
        let MulSigScriptRecord {
            mut args,
            capacity,
            since,
        } = record;
        args.extend(since.to_le_bytes().into_iter());
        let entry = map.entry(args.clone()).or_insert_with(Capacity::zero);

        *entry = entry.safe_add(capacity).unwrap_or_else(|e| {
            eprintln!("Warn: record capacity reduce overflow: {}", e);
            exit(1);
        });
    }
}

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

fn convert_lock_record(record: LockRecord, target: u64) -> Result<MulSigScriptRecord, Error> {
    let address = Address::from_str(&record.address)?;
    let dt = parse_date(&record.lock);
    let since = Outset.since_epoch(&dt, target);
    let mut args = Bytes::from(vec![0u8, 0, 1, 1]);
    args.extend(address.args);

    Ok(MulSigScriptRecord {
        args,
        capacity: Capacity::shannons(record.capacity * BYTE_SHANNONS),
        since,
    })
}
