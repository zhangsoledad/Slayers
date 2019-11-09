use crate::address::{Address, MAINNET_PREFIX};
use crate::input::{serialize_multisig_lock_args, LockRecord, RawRecord};
use crate::MULTISIG_CODE_HASH;
use bech32::{self, ToBase32};
use ckb_types::{bytes::Bytes, H256};
use failure::Error;
use serde_derive::Serialize;
use std::convert::{TryFrom, TryInto};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct Output {
    pub address: String,
    pub capacity: u64,
    pub mainnet_address: String,
}

#[derive(Debug, Serialize)]
pub struct LockOutput {
    pub address: String,
    pub capacity: u64,
    pub lock: Option<String>,
    pub mainnet_address: String,
}

impl TryFrom<RawRecord> for Output {
    type Error = Error;

    fn try_from(record: RawRecord) -> Result<Self, Self::Error> {
        let RawRecord { address, capacity } = record;
        let decode_address = Address::from_str(&address)?;
        Ok(Output {
            address,
            capacity,
            mainnet_address: decode_address.mainnet_short_format()?,
        })
    }
}

fn full_payload_format(args: Bytes) -> Result<String, Error> {
    let mut payload = vec![4u8]; // Type
    payload.extend_from_slice(H256::from_str(MULTISIG_CODE_HASH).unwrap().as_bytes());
    payload.extend_from_slice(&args);
    bech32::encode(MAINNET_PREFIX, payload.to_base32()).map_err(Into::into)
}

fn convert_lock_output(record: LockRecord, target: u64) -> Result<LockOutput, Error> {
    let LockRecord {
        address,
        capacity,
        lock,
    } = record;

    if let Some(ref date) = &lock {
        let args = serialize_multisig_lock_args(&address, date, target)?;
        let mainnet_address = full_payload_format(args)?;
        Ok(LockOutput {
            address,
            capacity,
            lock,
            mainnet_address,
        })
    } else {
        let decode_address = Address::from_str(&address)?;

        Ok(LockOutput {
            address,
            capacity,
            lock,
            mainnet_address: decode_address.mainnet_short_format()?,
        })
    }
}

pub fn write_incentives_output<P: AsRef<Path>, R: IntoIterator<Item = RawRecord>>(
    path: P,
    records: R,
) -> Result<(), Error> {
    let mut wtr = csv::Writer::from_path(path)?;

    for record in records
        .into_iter()
        .filter_map(|record| TryInto::<Output>::try_into(record).ok())
    {
        wtr.serialize(record)?;
    }
    Ok(())
}

pub fn write_allocate_output<P: AsRef<Path>, R: IntoIterator<Item = LockRecord>>(
    path: P,
    records: R,
    target: u64,
) -> Result<(), Error> {
    let mut wtr = csv::Writer::from_path(path)?;

    for record in records
        .into_iter()
        .filter_map(|record| convert_lock_output(record, target).ok())
    {
        wtr.serialize(record)?;
    }
    Ok(())
}
