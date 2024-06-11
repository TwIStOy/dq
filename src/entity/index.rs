use serde::{Deserialize, Serialize};

use crate::context::Context;

use super::Docset;

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexEntry {
    pub name: String,
    pub path: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexType {
    pub name: String,
    pub count: u32,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    pub entries: Vec<IndexEntry>,
    pub types: Vec<IndexType>,
}
