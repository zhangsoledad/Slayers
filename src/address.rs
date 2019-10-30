use bech32::{self, FromBase32};
use ckb_types::{core::ScriptHashType, packed::Script, prelude::*, H160, H256};
use failure::{Error, Fail};
use std::fmt;

const PREFIX: &str = "ckt";

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct Address {
    pub args: H160,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Fail)]
pub struct AddressError(String);

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Address {
    pub fn new(args: H160) -> Address {
        Address { args }
    }

    pub fn from_str(input: &str) -> Result<Address, Error> {
        let (hrp, data) = bech32::decode(input)?;
        let data = Vec::<u8>::from_base32(&data)?;
        if hrp != PREFIX {
            return Err(AddressError(format!("Invalid address hrp: {}", hrp)).into());
        }
        if data.len() == 22 {
            if data[0] != 0x01 {
                // short version for locks with popular code_hash
                return Err(AddressError(format!("Invalid address type: {}", data[0])).into());
            }
            if data[1] != 0x00 {
                // SECP256K1 + blake160
                return Err(AddressError(format!("Invalid code hash index: {}", data[1])).into());
            }
            H160::from_slice(&data[2..22])
                .map(Address::new)
                .map_err(Into::into)
        } else if data.len() == 25 {
            if &data[0..5] != b"\x01P2PH" {
                return Err(AddressError(format!("Invalid format type: {:?}", &data[0..5])).into());
            }
            H160::from_slice(&data[5..25])
                .map(Address::new)
                .map_err(Into::into)
        } else {
            Err(AddressError(format!("Invalid Address data length: {}", data.len())).into())
        }
    }
}
