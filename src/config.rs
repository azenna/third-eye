use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct FeedConfig {
    pub url: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub feeds: BTreeMap<String, FeedConfig>,
}

