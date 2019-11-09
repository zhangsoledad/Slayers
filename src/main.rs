mod address;
mod date;
mod explorer;
mod input;
mod output;
mod rpc;
mod template;

use crate::address::Address;
use ckb_chain_spec::ChainSpec;
use ckb_types::{
    bytes::Bytes,
    core::{capacity_bytes, Capacity},
};
use clap::{load_yaml, value_t, App};
use explorer::Explorer;
use input::{
    collect_allocate, parse_mining_competition_record, read_allocate,
    read_mining_competition_record, serialize_multisig_lock_args,
};
use output::{write_allocate_output, write_incentives_output};
use std::collections::BTreeMap;
use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::exit;
use template::{IssuedCell, Spec};
use tinytemplate::TinyTemplate;

static TEMPLATE: &str = include_str!("spec.toml.tt");
const DEFAULT_CODE_HASH: &str =
    "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8";
const MULTISIG_CODE_HASH: &str =
    "0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8";
const DEFAULT_TARGET_EPOCH: u64 = 89;
const MINING_COMPETITION_REWARD: Capacity = capacity_bytes!(168_000_000); // 0.5%
const FOUNDATION_RESERVE: Capacity = capacity_bytes!(672_000_000); // 2%
const INCENTIVES_ADDRESS: &str = "ckb1qyqy6mtud5sgctjwgg6gydd0ea05mr339lnslczzrc";
const FOUNDATION_ADDRESS: &str = "ckb1qyqyz340d4nhgtx2s75mp5wnavrsu7j5fcwqktprrp";
const FOUNDATION_LOCK: &str = "2020-07-01";
const INITIAL_ISSUES: Capacity = capacity_bytes!(33_600_000_000);

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let url = matches
        .value_of("url")
        .unwrap_or_else(|| "http://localhost:8114");
    let target = value_t!(matches, "target", u64).unwrap_or(DEFAULT_TARGET_EPOCH);
    let output_path = if matches.is_present("output") {
        let path = ::std::env::current_dir().unwrap().join("output");
        ::std::fs::create_dir_all(&path).unwrap();
        Some(path)
    } else {
        None
    };

    let foundation_reserve = foundation_reserve(target);
    let allocate = reduce_allocate(target, &output_path);

    let mut records = BTreeMap::new();
    load_mining_competition_records(&mut records, &output_path);
    let explorer = Explorer::new(url, target);
    let (timestamp, compact_target, message, epoch_length) =
        explorer.collect(&mut records).unwrap_or_else(|e| {
            eprintln!("explorer error: {}", e);
            exit(1);
        });
    let testnet_incentives = reduce_mining_competition_records(records);

    let context = Spec {
        timestamp,
        compact_target: format!("0x{:x}", compact_target),
        message: format!("{:x}", message),
        epoch_length,
        allocate,
        foundation_reserve: Some(foundation_reserve),
        testnet_incentives,
    };

    let mut tt = TinyTemplate::new();
    tt.add_template("lina", TEMPLATE).unwrap();
    let rendered = tt.render("lina", &context).unwrap();

    let spec: ChainSpec = toml::from_str(&rendered).unwrap();
    let consensus = spec.build_consensus().unwrap();

    let issued = consensus.genesis_block().transactions()[0]
        .outputs_capacity()
        .unwrap();
    assert_eq!(
        issued, INITIAL_ISSUES,
        "initial issued must be 33_600_000_000"
    );

    write_file(
        rendered,
        format!("0x{:x}", consensus.genesis_block().hash()),
    );
}

fn write_file(spec: String, hash: String) {
    fs::write("lina.toml", spec).unwrap();
    println!("spec: lina.toml");
    println!("hash: {}", hash);
    fs::write("hash.txt", hash).unwrap();
}

fn reduce_allocate(target: u64, output_path: &Option<PathBuf>) -> Vec<IssuedCell> {
    let allocate = include_bytes!("input/genesis_final.csv");
    let reader = BufReader::new(&allocate[..]);
    let records = read_allocate(reader).unwrap();
    if let Some(path) = output_path.as_ref() {
        write_allocate_output(path.join("genesis_final.csv"), records.clone(), target).unwrap();
    }
    collect_allocate(records, target)
}

#[rustfmt::skip]
fn load_mining_competition_records(map: &mut BTreeMap<Bytes, Capacity>, output_path: &Option<PathBuf>) {
    let prelude = [
        ("round1.csv",         include_str!("input/round1.csv")),
        ("round2.epoch.csv",   include_str!("input/round2.epoch.csv")),
        ("round2.mining.csv",  include_str!("input/round2.mining.csv")),
        ("round3.epoch.csv",   include_str!("input/round3.epoch.csv")),
        ("round3.mining.csv",  include_str!("input/round3.mining.csv")),
        ("round4.csv",         include_str!("input/round4.csv")),
        ("round5.stage1.csv",  include_str!("input/round5.stage1.csv")),
        ("round5.stage2.csv",  include_str!("input/round5.stage2.csv")),
    ];

    for (name, data) in prelude.iter() {
        let reader = BufReader::new(data.as_bytes());
        let records = read_mining_competition_record(reader).unwrap();

        if let Some(path) = output_path.as_ref() {
            write_incentives_output(path.join(*name), records.clone()).unwrap();
        }
        parse_mining_competition_record(records, map).unwrap();
    }
}

fn foundation_reserve(target: u64) -> IssuedCell {
    let dummy = Spec {
        timestamp: 0,
        compact_target: "0x1c00e904".to_string(),
        message: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        epoch_length: 1000,
        allocate: vec![],
        foundation_reserve: None,
        testnet_incentives: vec![],
    };

    let mut tt = TinyTemplate::new();
    tt.add_template("dummy", TEMPLATE).unwrap();
    let rendered = tt.render("dummy", &dummy).unwrap();

    let mut spec: ChainSpec = toml::from_str(&rendered).unwrap();
    // clean issued_cells
    spec.genesis.issued_cells = vec![];

    let consensus = spec.build_consensus().unwrap();

    let occupied = consensus.genesis_block().transactions()[0]
        .outputs_capacity()
        .unwrap();

    let foundation_reserve = FOUNDATION_RESERVE.safe_sub(occupied).unwrap();

    let args = serialize_multisig_lock_args(FOUNDATION_ADDRESS, FOUNDATION_LOCK, target).unwrap();

    IssuedCell {
        capacity: foundation_reserve.as_u64(),
        code_hash: MULTISIG_CODE_HASH.to_string(),
        args: format!("0x{}", faster_hex::hex_string(&args[..]).unwrap()),
    }
}

fn reduce_mining_competition_records(map: BTreeMap<Bytes, Capacity>) -> Vec<IssuedCell> {
    let total = map
        .iter()
        .map(|(_, capacity)| *capacity)
        .try_fold(Capacity::zero(), Capacity::safe_add)
        .unwrap_or_else(|_| {
            exit(1);
        });

    let mut issued: Vec<_> = map
        .into_iter()
        .map(|(args, capacity)| IssuedCell {
            capacity: capacity.as_u64(),
            code_hash: DEFAULT_CODE_HASH.to_string(),
            args: format!("0x{}", faster_hex::hex_string(&args[..]).unwrap()),
        })
        .collect();

    let remain = MINING_COMPETITION_REWARD
        .safe_sub(total)
        .unwrap_or_else(|_| {
            exit(1);
        });

    let incentives_address = Address::from_str(INCENTIVES_ADDRESS).unwrap();
    issued.push(IssuedCell {
        capacity: remain.as_u64(),
        code_hash: DEFAULT_CODE_HASH.to_string(),
        args: format!(
            "0x{}",
            faster_hex::hex_string(&incentives_address.args[..]).unwrap()
        ),
    });

    issued
}
