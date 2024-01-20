use serde::{Deserialize, Serialize};
use ratatui::widgets::Widget;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, List};
use crate::ast::Ast;

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

impl Into<Ast> for Rss {
    fn into(self) -> Ast{
        Ast::Block(self.channel.title, Box::new(
            Ast::List(self.channel.item.iter().map(|item| {
                format!("* {}: {}", item.title, item.link)
            }).collect())
        ))
    }
}
