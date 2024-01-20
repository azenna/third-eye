use crate::{
    config::Config,
    rss::Rss,
    tui::{Event, Tui},
    ast::Ast,
};
use crossterm::event::KeyCode;

use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use ratatui::{
    prelude::{Color, Frame, Style},
    widgets::{Block, Borders, List, ListState},
};

#[derive(Debug)]
pub enum Message {
    Down,
    Up,
    Back,
    SetFeed,
    GotFeed(Rss),
}

#[derive(Debug)]
pub enum Screen {
    Home,
    Loading,
    Content(Ast),
}

#[derive(Debug)]
pub struct App {
    pub config: Config,
    pub msg_tx: UnboundedSender<Message>,
    pub msg_rx: UnboundedReceiver<Message>,
    pub feed_state: ListState,
    pub screen: Screen,
}

impl App {
    pub fn new(config: Config) -> Self {
        let (msg_tx, msg_rx) = mpsc::unbounded_channel::<Message>();
        App {
            config,
            msg_tx,
            msg_rx,
            feed_state: ListState::default().with_selected(Some(0)),
            screen: Screen::Home,
        }
    }
    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut tui = Tui::new()?;
        tui.enter()?;

        loop {
            if let Some(evt) = tui.next().await {
                match evt {
                    Event::Render => {
                        tui.draw(|frame| self.view(frame))?;
                    }
                    Event::Key(key) => {
                        let msg = match key.code {
                            KeyCode::Char('k') => Message::Up,
                            KeyCode::Char('j') => Message::Down,
                            KeyCode::Enter => Message::SetFeed,
                            KeyCode::Char('b') => Message::Back,
                            KeyCode::Char('q') => break,
                            _ => continue,
                        };
                        self.msg_tx.send(msg)?;
                    }
                    Event::Quit => break,
                    Event::Error => {}
                    Event::Tick => {}
                }
            }
            while let Ok(msg) = self.msg_rx.try_recv() {
                self.handle_message(msg);
            }
        }
        tui.exit()?;
        Ok(())
    }
    pub fn handle_message(&mut self, msg: Message) {
        use Message::*;
        let selected = self.feed_state.selected_mut().as_mut().unwrap();
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
            SetFeed => {
                let feed: String = self
                    .config
                    .feeds
                    .keys()
                    .into_iter()
                    .nth(self.feed_state.selected().unwrap())
                    .unwrap()
                    .into();

                let feed_conf = self.config.feeds.get(&feed).cloned().unwrap();
                self.screen = Screen::Loading;

                let _msg_tx = self.msg_tx.clone();

                tokio::spawn(async move {
                    let resp = reqwest::get(&feed_conf.url)
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap();

                    let rss = serde_xml_rs::from_str(&resp).unwrap();
                    let _ = _msg_tx.send(GotFeed(rss));
                });
            }
            GotFeed(rss) => {
                self.screen = Screen::Content(rss.into());
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
            Content(ref rss) => {
                f.render_widget(rss, area);
            },
            Loading => {
                let load = Block::default().title("Loading");
                f.render_widget(load, area);
            }
        }
    }
}
