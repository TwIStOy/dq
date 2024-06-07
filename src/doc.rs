use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DocsEntry {
    pub name: String,
    pub slug: String,
    pub r#type: String,
    pub links: HashMap<String, String>,
    version: String,
    release: String,
    mtime: i64,
    db_size: i64,
}
