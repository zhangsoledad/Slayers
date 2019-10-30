use crate::address::Address;
use ckb_types::{core::Capacity, H160};
use failure::Error;
use serde_derive::Deserialize;
use std::convert::{TryFrom, TryInto};
use std::io::Read;

pub struct H264([u8; 33]);

#[derive(Debug, Deserialize)]
pub struct RawRecord1 {
    pub address: String,
    pub capacity: u64,
}

#[derive(Debug, Deserialize)]
pub struct RawRecord2 {
    pub pubkey: String,
    pub since: u64,
    pub capacity: u64,
}

#[derive(Debug, Deserialize)]
pub struct RawRecord3 {
    pub pubkeys: String,
    pub require_first_n: u64,
    pub threshold: u64,
    pub capacity: u64,
}

#[derive(Debug, Deserialize)]
pub struct RawRecord4 {
    pub pubkeys: String,
    pub require_first_n: u64,
    pub threshold: u64,
    pub since: u64,
    pub capacity: u64,
}

pub struct SigScriptRecord {
    pub args: H160,
    pub capacity: Capacity,
}

pub struct MulSigScriptRecord {
    pub pubkeys: Vec<H264>,
    pub require_first_n: u64,
    pub threshold: u64,
    pub since: u64,
    pub capacity: Capacity,
}

pub fn parse_record1<R: Read>(reader: R) -> Vec<SigScriptRecord> {
    let mut rdr = csv::Reader::from_reader(reader);
    rdr.deserialize()
        .filter_map(|record: Result<RawRecord1, _>| record.ok().and_then(|r| r.try_into().ok()))
        .collect()
}

impl TryFrom<RawRecord1> for SigScriptRecord {
    type Error = Error;

    fn try_from(record: RawRecord1) -> Result<Self, Self::Error> {
        let address = Address::from_str(&record.address)?;
        Ok(SigScriptRecord {
            args: address.args,
            capacity: Capacity::shannons(record.capacity),
        })
    }
}
