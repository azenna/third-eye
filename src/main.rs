use nu_ansi_term::Color::{Blue, Purple, Red, Yellow};
use serde::{Deserialize, Serialize};
use std::io::stdout;
use std::{collections::HashMap, fmt};

use anyhow::Context;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Stylize, Terminal, Modifier, Style, Frame, Color},
    widgets::{Paragraph, List, ListItem, Borders, Block, ListState}
};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    feeds: HashMap<String, FeedConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FeedConfig {
    url: String,
    title: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    title: String,
    link: String,
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}: {}",
            Yellow.bold().paint("*"),
            Blue.bold().paint(&self.title),
            Red.underline().paint(&self.link)
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Feed {
    title: String,
    item: Vec<Item>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Rss {
    channel: Feed,
}

#[derive(Debug)]
enum Screen {
    Home,
    Feed(String),
}

#[derive(Debug)]
struct Model {
    feeds: HashMap<String, FeedConfig>,
    feed_state: ListState,
    screen: Screen,
}

impl Model {
    pub fn new(feeds: HashMap<String, FeedConfig>) -> Self{
        Model {
            feeds,
            feed_state: ListState::default().with_selected(Some(0)),
            screen: Screen::Home,
        }
    }
    pub fn handle_message(&mut self, msg: Message){
        use Message::*;
        let selected = self.feed_state
            .selected_mut()
            .as_mut()
            .unwrap();
        let count = self.feeds.len();

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
                self.screen = Screen::Feed(feed);
            }
        }
    }

    pub fn view(&mut self, f: &mut Frame) {
        let area = f.size();

        use Screen::*;

        match self.screen {
            Home => {
                let list = List::new(self.feeds.keys().cloned())
                    .block(Block::default().title("Swarms").borders(Borders::ALL))
                    .highlight_style(Style::default().bg(Color::White));

                f.render_stateful_widget(list, area, &mut self.feed_state);
            }
            Feed(ref feed) => {
                let mut feed_block = Block::default().title(feed.as_str()).borders(Borders::ALL);
                let feed_conf = self.feeds.get(feed).unwrap();
                let rss = handle_feed(&feed_conf);
                let key_text = Paragraph::new(app.key_input.clone()).block(key_block);
            }
        }

    }
}

enum Message {
    Down,
    Up,
    SetFeed(String),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // Config parsing
    let toml_string =
        std::fs::read_to_string("third_eye.toml").context("Couldn't open 'third_eye.toml'")?;
    let config: Config = toml::from_str(&toml_string).context("Couldn't parse config")?;

    // Tui
    let mut model = Model::new(config.feeds);

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    loop {
        terminal.draw(|frame| model.view(frame))?;

        if event::poll(std::time::Duration::from_millis(250))? {
           if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let msg = match key.code {
                        KeyCode::Char('k') => Message::Up,
                        KeyCode::Char('j') => Message::Down,
                        KeyCode::Enter => Message::SetFeed(model.feeds
                            .keys()
                            .into_iter()
                            .nth(model.feed_state.offset())
                            .unwrap()
                            .into()),
                        KeyCode::Char('q') => break,
                        _ => continue,
                    };
                    model.handle_message(msg);
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
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
