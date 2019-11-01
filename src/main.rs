mod address;
mod explorer;
mod input;
mod rpc;
mod template;

use ckb_types::{bytes::Bytes, core::Capacity};
use clap::{load_yaml, value_t, App};
use explorer::Explorer;
use input::parse_mining_competition_record;
use std::collections::BTreeMap;
use std::io::BufReader;
use std::process::exit;
use template::{IssuedCell, Spec};
use tinytemplate::TinyTemplate;

static TEMPLATE: &str = include_str!("spec.toml");
const SIG_CODE_HASH: &str = "0x";
const TARGET_EPOCH: u64 = 89;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let mut records = BTreeMap::new();
    load_mining_competition_records(&mut records);

    let url = matches
        .value_of("url")
        .unwrap_or_else(|| "http://127.0.0.1:8114");
    let target = value_t!(matches, "target", u64).unwrap_or(TARGET_EPOCH);
    let explorer = Explorer::new(url, target);
    let (timestamp, message) = explorer.collect(&mut records).unwrap_or_else(|e| {
        eprintln!("explorer error: {}", e);
        exit(1);
    });

    let issued_cells = reduce(records);

    let context = Spec {
        timestamp,
        compact_target: "0x1".to_string(),
        message: format!("{:x}", message),
        issued_cells,
    };

    let mut tt = TinyTemplate::new();
    tt.add_template("lina", TEMPLATE).unwrap();
    let rendered = tt.render("lina", &context).unwrap();
    println!("{}", rendered);
}

fn load_mining_competition_records(map: &mut BTreeMap<Bytes, Capacity>) {
    {
        let round1 = include_bytes!("input/round1.csv");
        let reader = BufReader::new(&round1[..]);
        parse_mining_competition_record(reader, map);
    }

    {
        let round2_epoch = include_bytes!("input/round2.epoch.csv");
        let reader = BufReader::new(&round2_epoch[..]);
        parse_mining_competition_record(reader, map);
    }

    {
        let round2_mininng = include_bytes!("input/round2.mining.csv");
        let reader = BufReader::new(&round2_mininng[..]);
        parse_mining_competition_record(reader, map);
    }

    {
        let round3_epoch = include_bytes!("input/round3.epoch.csv");
        let reader = BufReader::new(&round3_epoch[..]);
        parse_mining_competition_record(reader, map);
    }

    {
        let round3_mininng = include_bytes!("input/round3.mining.csv");
        let reader = BufReader::new(&round3_mininng[..]);
        parse_mining_competition_record(reader, map);
    }

    {
        let round4 = include_bytes!("input/round4.csv");
        let reader = BufReader::new(&round4[..]);
        parse_mining_competition_record(reader, map);
    }

    {
        let round5_stage1 = include_bytes!("input/round5.stage1.csv");
        let reader = BufReader::new(&round5_stage1[..]);
        parse_mining_competition_record(reader, map);
    }
}

fn reduce(map: BTreeMap<Bytes, Capacity>) -> Vec<IssuedCell> {
    map.into_iter()
        .map(|(args, capacity)| IssuedCell {
            capacity: capacity.as_u64(),
            code_hash: SIG_CODE_HASH.to_string(),
            args: format!("0x{}", faster_hex::hex_string(&args[..]).unwrap()),
            hash_type: "type".to_string(),
        })
        .collect()
}
