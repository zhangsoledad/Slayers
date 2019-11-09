use crate::{
    address::Address,
    date::{parse_date, Outset},
    template::IssuedCell,
    DEFAULT_CODE_HASH, MULTISIG_CODE_HASH,
};
use ckb_types::{bytes::Bytes, core::Capacity};
use failure::Error;
use serde_derive::Deserialize;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::io::Read;

const BYTE_SHANNONS: u64 = 100_000_000;

#[derive(Debug, Clone, Deserialize)]
pub struct RawRecord {
    pub address: String,
    pub capacity: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LockRecord {
    pub address: String,
    pub capacity: u64,
    pub lock: Option<String>,
}

pub struct TestnetIncentives {
    pub args: Bytes,
    pub capacity: Capacity,
}

pub struct Allocate {
    pub args: Bytes,
    pub code_hash: String,
    pub capacity: Capacity,
}

pub fn read_allocate<R: Read>(reader: R) -> Result<Vec<LockRecord>, Error> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(reader);
    let records: Result<Vec<LockRecord>, csv::Error> = rdr.deserialize().collect();
    records.map_err(Into::into)
}

pub fn collect_allocate<R: IntoIterator<Item = LockRecord>>(
    records: R,
    target: u64,
) -> Vec<IssuedCell> {
    records
        .into_iter()
        .filter_map(|record| convert_record_allocate(record, target).ok())
        .map(|record| {
            let Allocate {
                args,
                code_hash,
                capacity,
            } = record;
            IssuedCell {
                capacity: capacity.as_u64(),
                code_hash,
                args: format!("0x{}", faster_hex::hex_string(&args[..]).unwrap()),
            }
        })
        .collect()
}

pub fn read_mining_competition_record<R: Read>(reader: R) -> Result<Vec<RawRecord>, Error> {
    let mut rdr = csv::Reader::from_reader(reader);
    let records: Result<Vec<RawRecord>, csv::Error> = rdr.deserialize().collect();
    records.map_err(Into::into)
}

pub fn parse_mining_competition_record<R: IntoIterator<Item = RawRecord>>(
    records: R,
    map: &mut BTreeMap<Bytes, Capacity>,
) -> Result<(), Error> {
    let records: Vec<_> = records
        .into_iter()
        .filter_map(|record| record.try_into().ok())
        .collect();

    for record in records {
        let TestnetIncentives { args, capacity } = record;
        let entry = map.entry(args.clone()).or_insert_with(Capacity::zero);
        *entry = entry.safe_add(capacity)?;
    }
    Ok(())
}

impl TryFrom<RawRecord> for TestnetIncentives {
    type Error = Error;

    fn try_from(record: RawRecord) -> Result<Self, Self::Error> {
        let address = Address::from_str(&record.address)?;
        Ok(TestnetIncentives {
            args: address.args,
            capacity: Capacity::shannons(record.capacity * BYTE_SHANNONS),
        })
    }
}

pub fn blake160(message: &[u8]) -> Bytes {
    Bytes::from(&ckb_hash::blake2b_256(message)[..20])
}

pub fn serialize_multisig_lock_args(
    address: &str,
    date: &str,
    target: u64,
) -> Result<Bytes, Error> {
    let address = Address::from_str(address)?;
    let dt = parse_date(date)?;
    let since = Outset.since_epoch(&dt, target);
    let mut script = Bytes::from(vec![0u8, 0, 1, 1]);
    script.extend_from_slice(&address.args);
    let mut args = blake160(&script).to_vec();

    args.extend(since.to_le_bytes().iter());
    Ok(Bytes::from(args))
}

pub fn convert_record_allocate(record: LockRecord, target: u64) -> Result<Allocate, Error> {
    if let Some(ref date) = &record.lock {
        let args = serialize_multisig_lock_args(&record.address, date, target)?;
        Ok(Allocate {
            args,
            code_hash: MULTISIG_CODE_HASH.to_string(),
            capacity: Capacity::shannons(record.capacity * BYTE_SHANNONS),
        })
    } else {
        let address = Address::from_str(&record.address)?;
        Ok(Allocate {
            args: address.args,
            code_hash: DEFAULT_CODE_HASH.to_string(),
            capacity: Capacity::shannons(record.capacity * BYTE_SHANNONS),
        })
    }
}
