use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub title: String,
    pub link: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Feed {
    pub title: String,
    pub item: Vec<Item>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rss {
    pub channel: Feed,
}
