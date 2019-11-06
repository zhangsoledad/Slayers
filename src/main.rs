mod address;
mod date;
mod explorer;
mod input;
mod rpc;
mod template;

use ckb_types::{
    bytes::Bytes,
    core::{capacity_bytes, Capacity},
};
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
const MINING_COMPETITION_REWARD: Capacity = capacity_bytes!(168_000_000); // 0.5%

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
    let (timestamp, compact_target, message, epoch_length) =
        explorer.collect(&mut records).unwrap_or_else(|e| {
            eprintln!("explorer error: {}", e);
            exit(1);
        });

    let issued_cells = reduce_mining_competition_records(records);

    // println!("timestamp {}", timestamp);
    // println!("compact_target {:x}", compact_target);
    // println!("message {:x}", message);
    // println!("epoch_length {}", epoch_length);

    let context = Spec {
        timestamp,
        compact_target: format!("0x{:x}", compact_target),
        message: format!("{:x}", message),
        epoch_length,
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

fn reduce_mining_competition_records(map: BTreeMap<Bytes, Capacity>) -> Vec<IssuedCell> {
    let total = map
        .iter()
        .map(|(_, capacity)| *capacity)
        .try_fold(Capacity::zero(), Capacity::safe_add)
        .unwrap_or_else(|e| {
            exit(1);
        });

    let mut issued: Vec<_> = map
        .into_iter()
        .map(|(args, capacity)| IssuedCell {
            capacity: capacity.as_u64(),
            code_hash: SIG_CODE_HASH.to_string(),
            args: format!("0x{}", faster_hex::hex_string(&args[..]).unwrap()),
            hash_type: "type".to_string(),
        })
        .collect();

    let remain = MINING_COMPETITION_REWARD
        .safe_sub(total)
        .unwrap_or_else(|e| {
            exit(1);
        });
    // println!("remain {}", remain.as_u64());

    issued.push(IssuedCell {
        capacity: remain.as_u64(),
        code_hash: SIG_CODE_HASH.to_string(),
        args: "0x000000".to_string(),
        hash_type: "type".to_string(),
    });

    issued
}
