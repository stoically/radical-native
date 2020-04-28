use serde::{Deserialize, Serialize};
use serde_json::Value;
use seshat::Profile;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResults {
    pub count: usize,
    pub results: Vec<SearchResult>,
    pub highlights: Vec<SearchHighlight>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub rank: f32,
    pub result: String,
    pub context: SearchResultContext,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultContext {
    pub events_before: Vec<Value>,
    pub events_after: Vec<Value>,
    pub profile_info: HashMap<String, Profile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchHighlight {}
