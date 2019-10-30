mod address;
mod aggregate;
mod explorer;
mod input;
mod rpc;
mod template;

use aggregate::reduce_sig_record;
use clap::{load_yaml, value_t, App};
use input::parse_record1;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use template::{IssuedCell, Spec};
use tinytemplate::TinyTemplate;

static TEMPLATE: &'static str = include_str!("spec.toml");

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let records = value_t!(matches.value_of("records"), PathBuf).unwrap_or_else(|e| e.exit());
    println!("records path: {:?}", records);

    let issued_cells = get_records(records);
    println!("issued_cells len: {:?}", issued_cells.len());

    let mut tt = TinyTemplate::new();
    tt.add_template("lina", TEMPLATE).unwrap();

    let context = Spec {
        timestamp: 0,
        compact_target: "0x1".to_string(),
        message: "test".to_string(),
        issued_cells: issued_cells,
    };

    let rendered = tt.render("lina", &context).unwrap();
    println!("{}", rendered);
}

fn get_records<P: AsRef<Path>>(path: P) -> Vec<IssuedCell> {
    let f = File::open(path).unwrap_or_else(|e| panic!("Error: {}", e.description()));
    let reader = BufReader::new(f);
    let record = parse_record1(reader);
    println!("record len: {:?}", record.len());
    reduce_sig_record(record)
}
