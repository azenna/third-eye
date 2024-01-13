use crossterm::{
    cursor,
    event::{Event as CrosstermEvent, KeyEvent, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{stdout, Stdout},
    ops::{Deref, DerefMut},
    time::Duration,
};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
pub enum Event {
    Quit,
    Error,
    Tick,
    Render, // new
    Key(KeyEvent),
}

pub struct Tui {
    pub backend: Terminal<CrosstermBackend<Stdout>>,
    pub event_rx: mpsc::UnboundedReceiver<Event>,
    pub event_tx: mpsc::UnboundedSender<Event>,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub task: JoinHandle<()>,
    cancellation_token: CancellationToken,
}

impl Tui {
    pub fn new() -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::unbounded_channel();

        Ok(Self {
            backend: Terminal::new(CrosstermBackend::new(stdout()))?,
            event_rx: rx,
            event_tx: tx,
            tick_rate: 4.0,
            frame_rate: 60.0,
            task: tokio::spawn(async {}),
            cancellation_token: CancellationToken::new(),
        })
    }
    pub fn start(&mut self) {
        let render_delay = Duration::from_secs_f64(1.0 / self.frame_rate);
        let tick_delay = Duration::from_secs_f64(1.0 / self.tick_rate);
        let _event_tx = self.event_tx.clone();

        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let _cancel_token = self.cancellation_token.clone();

        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();

            let mut render_interval = tokio::time::interval(render_delay);
            let mut tick_interval = tokio::time::interval(tick_delay);
            loop {
                let tick_tick = tick_interval.tick();
                let render_tick = render_interval.tick();
                let event = reader.next().fuse();

                tokio::select! {
                    _ = _cancel_token.cancelled() => {
                        break;
                    }
                    maybe_event = event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                if let CrosstermEvent::Key(key) = evt {
                                    if key.kind == KeyEventKind::Press {
                                        _event_tx.send(Event::Key(key)).unwrap();
                                    }
                                }
                            }
                            Some(Err(_)) => {
                                _event_tx.send(Event::Error).unwrap();
                            }
                            None => {}
                        }
                    }
                    _ = tick_tick => {
                        _event_tx.send(Event::Tick).unwrap();

                    }
                    _ = render_tick => {
                        _event_tx.send(Event::Render).unwrap();
                    }
                }
            }
        });
    }
    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.cancel();

        let mut counter = 0;
        while !self.task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;

            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                panic!("Failed to abort after 100ms");
            }
        }
        Ok(())
    }
    pub fn enter(&mut self) -> anyhow::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
        self.start();
        Ok(())
    }

    pub fn exit(&mut self) -> anyhow::Result<()> {
        self.stop()?;
        crossterm::execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }
    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<CrosstermBackend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.backend
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}
