use serde::{Deserialize, Serialize};
use ratatui::widgets::Widget;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, List};

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

impl Widget for &Rss {
    fn render(self, area: Rect, buf: &mut Buffer){
        let feed = &self.channel;
        let block = Block::default()
            .title(feed.title.as_str())
            .borders(Borders::ALL);
        List::new(
                feed.item
                .iter()
                .map(|item| format!("* {}: {}", item.title, item.link)),
        )
        .block(block).render(area, buf);
    }
}
