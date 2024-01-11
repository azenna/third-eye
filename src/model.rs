use crate::config::{Config, FeedConfig};
use crate::rss::Rss;

use ratatui::{
    prelude::{Style, Frame, Color},
    widgets::{List, Borders, Block, ListState}
};

use anyhow::Context;

#[derive(Debug)]
pub enum Message {
    Down,
    Up,
    Back,
    SetFeed(String),
}

#[derive(Debug)]
pub enum Screen {
    Home,
    Feed(Rss),
}

#[derive(Debug)]
pub struct Model {
    pub config: Config,
    pub feed_state: ListState,
    pub screen: Screen,
}

impl Model {
    pub fn new(config: Config) -> Self{
        Model {
            config,
            feed_state: ListState::default().with_selected(Some(0)),
            screen: Screen::Home,
        }
    }
    pub async fn handle_message(&mut self, msg: Message){
        use Message::*;
        let selected = self.feed_state
            .selected_mut()
            .as_mut()
            .unwrap();
        let count = self.config.feeds.len();

        match msg {
            Down => {
                if *selected + 1 >= count {
                    *selected = 0;
                } else {
                    *selected += 1;
                }
            }
            Up => {
                if (*selected as isize - 1) < 0 {
                    *selected = count - 1;
                } else {
                    *selected -= 1;
                }
            }
            SetFeed(feed) => {
                let feed_conf = self.config.feeds.get(&feed).unwrap();
                let rss = handle_feed(&feed_conf).await.unwrap();

                self.screen = Screen::Feed(rss);
            }
            Back => {
                self.screen = Screen::Home;
            }
        }
    }

    pub fn view(&mut self, f: &mut Frame) {
        use Screen::*;

        let area = f.size();
        match self.screen {
            Home => {
                let list = List::new(self.config.feeds.keys().cloned())
                    .block(Block::default().title("Feeds").borders(Borders::ALL))
                    .highlight_style(Style::default().bg(Color::White));

                f.render_stateful_widget(list, area, &mut self.feed_state);
            }
            Feed(ref feed) => {
                let feed_block = Block::default().title(feed.channel.title.as_str()).borders(Borders::ALL);
                let stories = List::new(feed.channel.item.iter().map(|item| {
                    format!("* {}: {}", item.title, item.link)
                })).block(feed_block);

                f.render_widget(stories, area);
            }
        }
    }
}

async fn handle_feed(config: &FeedConfig) -> anyhow::Result<Rss> {
    let resp = reqwest::get(&config.url)
        .await
        .context("Failed to fetch feed")?
        .text()
        .await
        .context("Couldn't turn feed to text")?;

    serde_xml_rs::from_str(&resp).context("Couldn't parse rss feed")
}
