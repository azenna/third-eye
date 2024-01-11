mod model;
mod config;
mod rss;

use model::{Message, Model};
use config::Config;

use std::io::stdout;

use anyhow::Context;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // Config parsing
    let toml_string =
        std::fs::read_to_string("third_eye.toml").context("Couldn't open 'third_eye.toml'")?;
    let config: Config = toml::from_str(&toml_string).context("Couldn't parse config")?;

    // Tui
    let mut model = Model::new(config);

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
                        KeyCode::Enter => Message::SetFeed(model.config.feeds
                            .keys()
                            .into_iter()
                            .nth(model.feed_state.selected().unwrap())
                            .unwrap()
                            .into()),
                        KeyCode::Char('b') => Message::Back,
                        KeyCode::Char('q') => break,
                        _ => continue,
                    };
                    model.handle_message(msg).await;
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
