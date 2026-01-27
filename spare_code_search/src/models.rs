use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryPoint {
    pub candidates: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Prediction {
    pub context: String,
    pub prefix: String,
    pub suffix: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataPoint {
    pub id: String,
    pub repo: String,
    pub revision: String,
    pub path: String,
    pub modified: Vec<String>,
    pub prefix: String,
    pub suffix: String,
    pub archive: String,
    pub completion_point: Option<(usize, usize)>, // (line, column)
    pub diff: Option<String>,
}
