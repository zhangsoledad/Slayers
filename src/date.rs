use chrono::{
    naive::NaiveDate,
    offset::{TimeZone, Utc},
    DateTime,
};
use ckb_types::core::EpochNumberWithFraction;
use failure::Error;

const EPOCH_DURATION: u64 = 4 * 60 * 60;
const EPOCH_LENGTH: u64 = 1_800;
const SINCE_FLAG: u64 = 0x2000_0000_0000_0000;

pub fn parse_date(input: &str) -> Result<DateTime<Utc>, Error> {
    let date = NaiveDate::parse_from_str(input, "%Y-%m-%d")?.and_hms(0, 0, 0);
    Ok(DateTime::from_utc(date, Utc))
}

pub struct Outset;

impl Outset {
    pub fn since(&self, date: &DateTime<Utc>) -> u64 {
        (date.timestamp() - Utc.ymd(2019, 11, 16).and_hms(6, 0, 0).timestamp()) as u64
    }

    pub fn since_epoch(&self, date: &DateTime<Utc>, target: u64) -> u64 {
        let since = self.since(date);
        let offset = (since / EPOCH_DURATION) + 89;
        let overflow = target > offset;
        let epoch = if !overflow { offset - target } else { 0 };
        let index = if !overflow {
            (since % EPOCH_DURATION) * 1800 / EPOCH_DURATION
        } else {
            0
        };
        EpochNumberWithFraction::new(epoch, index, EPOCH_LENGTH).full_value() + SINCE_FLAG
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        let dt = parse_date("2020-01-01");
        assert!(dt.is_ok(), "{:?}", dt);
        assert_eq!(dt.unwrap(), Utc.ymd(2020, 1, 1).and_hms(0, 0, 0));
    }
}
