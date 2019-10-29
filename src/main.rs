mod template;

use template::{Spec, IssuedCell};
use tinytemplate::TinyTemplate;

static TEMPLATE : &'static str = include_str!("spec.toml");


fn main() {
    let mut tt = TinyTemplate::new();
    tt.add_template("hello", TEMPLATE).unwrap();

    let context = Spec {
        timestamp: 0,
        compact_target: "0x1".to_string(),
        message: "test".to_string(),
        issued_cells: vec![],
    };

    let rendered = tt.render("hello", &context).unwrap();
    println!("{}", rendered);
}
