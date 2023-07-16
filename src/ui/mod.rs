mod layout;
use layout::banner;
use layout::confirm_popup::centered_rect;

use crate::start_bot;

use tokio::time::Duration;

use crossterm::event::{Event, KeyCode};
use std::{io, sync::{Arc, Mutex}};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear, Wrap},
    Frame, Terminal,
};

#[derive(Clone)]
pub enum InputMode {
    Normal,
    Updating,
}

/// App holds the state of the application
#[derive(Clone)]
pub struct App {
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    pub messages: Arc<Mutex<Vec<String>>>,

    show_confirm_popup: bool,

    confirm_popup_selection: Option<bool>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Arc::new(Mutex::new(Vec::new())),
            show_confirm_popup: false,
            confirm_popup_selection: None
        }
    }
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    /* let messages_tui = Arc::clone(&app.messages); */

    let (tx, rx) = std::sync::mpsc::channel::<crossterm::event::Event>();

    std::thread::spawn(move || {
        loop {
            if crossterm::event::poll(Duration::from_millis(100)).unwrap() {
                if let Ok(event) = crossterm::event::read() {
                    tx.send(event).unwrap();
                }
            }
        }
    });

    loop {
        terminal.draw(|f| {
            ui(f, &app);
        })?;

        let maybe_key_event = rx.recv_timeout(Duration::from_millis(100));
        let key_event = if let Ok(event) = maybe_key_event {
            Some(event)
        } else {
            None
        };

        match app.input_mode {
            InputMode::Normal => if let Some(Event::Key(event)) = key_event {
                match event.code {
                    KeyCode::Esc => {
                        if app.show_confirm_popup {
                            app.show_confirm_popup = false;
                        }
                    },
                    KeyCode::Char('s') => {            
                        if app.show_confirm_popup {
                            app.confirm_popup_selection = Some(false);
                        } else {
                            app.confirm_popup_selection = None;
                        }
                        app.show_confirm_popup = !app.show_confirm_popup;
                    },
                    KeyCode::Char('y') => {
                        if app.show_confirm_popup {
                            app.confirm_popup_selection = Some(true);
                            app.show_confirm_popup = false;
                            app.input_mode = InputMode::Updating;
                        
                            {
                                let messages = Arc::clone(&app.messages);
                                tokio::spawn(async move { start_bot(messages).await });
                            }
                        }   
                    }
                    KeyCode::Char('n') => {
                        if app.show_confirm_popup  {
                            app.confirm_popup_selection = Some(false);
                            app.show_confirm_popup = false;
                        }
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    },
                    _ => {},
                }
            },
            InputMode::Updating => if let Some(Event::Key(event)) = key_event {
                match event.code {
                    // KeyCode::Enter => {
                    //     let mut messages = messages_tui.lock().unwrap();
                    //     messages.push("[INFO] - Starting bot...".to_owned());
                    // },
                    KeyCode::Char('q') => {
                        return Ok(());
                    },
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    },
                    KeyCode::Backspace => {
                        app.input.pop();
                    },
                    // KeyCode::Esc => {
                    //     app.input_mode = InputMode::Normal;
                    // },
                    _ => {},
                }
            },
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(12),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let style = Style::default().add_modifier(Modifier::RAPID_BLINK);
    let mut title = Text::from(banner::BANNER);
    title.patch_style(style);
    let title_message = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL).title("Title"));
    f.render_widget(title_message, chunks[0]);

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("s", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start bot."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Updating => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit. If you want to restart the bot, you will have to restart the program.")
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help_message, chunks[2]);

    let messages_tui = Arc::clone(&app.messages);
    let messages = messages_tui.lock().unwrap();

    let messages: Vec<ListItem> = messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
            ListItem::new(content)
        })
        .collect();
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Bot logs:"));
    f.render_widget(messages, chunks[1]);

    if app.show_confirm_popup {
        let block = Block::default().borders(Borders::ALL);
        let area = centered_rect(20, 10, size);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(block, area);

        let popup_text = Paragraph::new(Span::styled(
            "Starting a Discord bot",
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        let popup_chunks = Layout::default()
            .constraints([Constraint::Percentage(45), Constraint::Percentage(50)].as_ref())
            .split(area);

        f.render_widget(popup_text, popup_chunks[0]);

        let selection_text = String::from("Continue running the bot? (y/n)");

        let selection_paragraph = Paragraph::new(Span::styled(
            selection_text,
            Style::default().fg(Color::Yellow),
        ))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        f.render_widget(selection_paragraph, popup_chunks[1]);
    }
}
