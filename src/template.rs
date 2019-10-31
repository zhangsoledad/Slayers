use serde_derive::Serialize;

#[derive(Debug, Serialize)]
pub struct Spec {
    pub timestamp: u64,
    pub compact_target: String,
    pub message: String,
    pub issued_cells: Vec<IssuedCell>,
}

#[derive(Debug, Serialize)]
pub struct IssuedCell {
    pub capacity: u64,
    pub code_hash: String,
    pub args: String,
    pub hash_type: String,
}
