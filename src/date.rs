use chrono::{
    offset::{TimeZone, Utc},
    DateTime,
};
use ckb_types::core::EpochNumberWithFraction;
use std::process::exit;

const EPOCH_DURATION: u64 = 4 * 60 * 60;
const EPOCH_LENGTH: u64 = 1_800;
const SINCE_FLAG: u64 = 0x2000_0000_0000_0000;

pub fn parse_date(input: &str) -> DateTime<Utc> {
    Utc.datetime_from_str(input, "%Y-%m-%d")
        .unwrap_or_else(|_| exit(0))
}

pub struct Outset;

impl Outset {
    pub fn since(&self, date: &DateTime<Utc>) -> u64 {
        (date.timestamp() - Utc.ymd(2019, 11, 16).and_hms(6, 0, 0).timestamp()) as u64
    }

    pub fn since_epoch(&self, date: &DateTime<Utc>, target: u64) -> u64 {
        let since = self.since(date);
        let epoch = (since / EPOCH_DURATION) - (target - 89);
        let index = (since % EPOCH_DURATION) * 1800 / EPOCH_DURATION;

        EpochNumberWithFraction::new(epoch, index, EPOCH_LENGTH).full_value() + SINCE_FLAG
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_date() {
        let dt = parse_date("2020-01-01");
        assert_eq!(dt, Utc.ymd(2020, 1, 1).and_hms(0, 0, 0));
    }
}
