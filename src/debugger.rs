// src/debugger.rs

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame, Terminal,
};
use tokio::sync::mpsc;

// ── Events ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct MessageEvent {
    pub id: u64,
    pub msg_type: u16,
    pub payload: Vec<u8>,
    pub latency_us: u64,
}

// ── App ──────────────────────────────────────────────────────────────────────

pub struct App {
    pub messages: Vec<MessageEvent>,
    pub selected: usize,
    pub total_messages: u64,
    pub bytes_received: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            selected: 0,
            total_messages: 0,
            bytes_received: 0,
        }
    }

    pub fn add_message(&mut self, msg: MessageEvent) {
        self.bytes_received += msg.payload.len() as u64;
        self.total_messages += 1;
        self.messages.push(msg);
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }

    pub fn next(&mut self) {
        if self.selected + 1 < self.messages.len() {
            self.selected += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
}

// ── UI ───────────────────────────────────────────────────────────────────────

pub fn draw_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(f.size());

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[0]);

    draw_messages(f, app, top[0]);
    draw_stats(f, app, top[1]);
    draw_detail(f, app, chunks[1]);
}

fn draw_messages(f: &mut Frame, app: &App, area: Rect) {
    let rows: Vec<Row> = app.messages.iter().enumerate().map(|(i, m)| {
        Row::new(vec![
            m.id.to_string(),
            m.msg_type.to_string(),
            m.payload.len().to_string(),
            format!("{:.2}µs", m.latency_us),
        ])
        .style(if i == app.selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        })
    }).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Min(10),
        ],
    )
    .header(Row::new(vec!["ID", "Type", "Size", "Latency"])
        .style(Style::default().fg(Color::Cyan)))
    .block(Block::default().title("Messages").borders(Borders::ALL));

    f.render_widget(table, area);
}

fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
    let text = format!("Total: {}\nBytes: {}", app.total_messages, app.bytes_received);
    let p = Paragraph::new(text)
        .block(Block::default().title("Stats").borders(Borders::ALL));
    f.render_widget(p, area);
}

fn draw_detail(f: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(msg) = app.messages.get(app.selected) {
        if msg.payload.is_ascii() {
            String::from_utf8_lossy(&msg.payload).to_string()
        } else {
            format!("[binary {} bytes]", msg.payload.len())
        }
    } else {
        "No message selected".to_string()
    };
    let p = Paragraph::new(content)
        .block(Block::default().title("Detail").borders(Borders::ALL));
    f.render_widget(p, area);
}

// ── Point d'entrée du débogueur ──────────────────────────────────────────────

pub async fn run_debugger() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let mut id = 0;
        loop {
            let msg = MessageEvent {
                id,
                msg_type: 1,
                payload: b"hello world".to_vec(),
                latency_us: 100,
            };
            tx.send(msg).await.unwrap();
            id += 1;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });

    let mut app = App::new();

    loop {
        while let Ok(msg) = rx.try_recv() {
            app.add_message(msg);
        }
        terminal.draw(|f| draw_ui(f, &app))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}