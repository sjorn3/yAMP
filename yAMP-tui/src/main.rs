#![allow(dead_code)] // todo remove on push

mod errors;
mod tui;

use crossterm::event::{self, Event, KeyCode};
use ratatui::buffer::*;
use ratatui::{prelude::*, widgets::*};
use std::time::Duration;

#[derive(Debug, Default)]
struct Model {
    draw: bool,
    counter: i32,
    running_state: RunningState,
    list: ListModel,
}

impl Model {
    fn initial() -> Self {
        Self {
            draw: true,
            ..Self::default()
        }
    }
}

#[derive(Debug, Default)]
struct ListModel {
    items: Vec<String>,
    selected: usize,
}

impl ListModel {
    fn new(items: Vec<String>) -> Self {
        Self { items, selected: 0 }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(PartialEq)]
enum Message {
    Increment,
    Decrement,
    Reset,
    Quit,
    List(ListMessage),
    Redraw,
}

#[derive(PartialEq)]
enum ListMessage {
    MoveUp,
    MoveDown,
}

#[derive(PartialEq)]
enum Drawing {
    Redraw,
    Noop,
}

fn main() -> music_cache::Result<()> {
    errors::install_hooks()?;
    let mut terminal = tui::init()?;
    let mut model = Model::initial();

    let mut current_msg = None;

    while model.running_state != RunningState::Done {
        if model.draw {
            terminal.draw(|f| view(&mut model, f))?;
            model.draw = false;
        }

        current_msg = if let Some(msg) = current_msg {
            update(&mut model, msg)
        } else {
            handle_event(&model)?
        }
    }

    tui::restore()?;
    Ok(())
}

fn view(model: &mut Model, f: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(30), Constraint::Percentage(100)])
        .split(f.size());

    f.render_widget(
        ListModel::new(vec!["hello".to_string(), "world".to_string()]),
        // .block(Block::new().borders(Borders::ALL)),
        layout[0],
    );
    f.render_widget(
        Paragraph::new(format!("Counter: {}", model.counter))
            .block(Block::new().borders(Borders::ALL)),
        layout[1],
    );
}

impl Widget for ListModel {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let line = Line::from(vec!["Hello, World!".red()]);
        let line1 = Line::from(vec!["Goodbye, World!".blue()]);
        let line2 = Line::from(vec!["Another line.".green().bold()]);
        buf.set_line(0, 0, &line, area.width);
        buf.set_line(0, 1, &line1, area.width);
        buf.set_line(0, 2, &line2, area.width);
    }
}

fn handle_event(_: &Model) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == event::KeyEventKind::Press {
                    return Ok(handle_key(key));
                }
            }
            Event::Resize(_, _) => {
                return Ok(Some(Message::Redraw));
            }
            _ => {}
        }
    }
    Ok(None)
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') => Some(Message::Increment),
        KeyCode::Char('k') => Some(Message::Decrement),
        KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Increment => {
            model.counter += 1;
            if model.counter > 50 {
                return Some(Message::Reset);
            }
        }
        Message::Decrement => {
            model.counter -= 1;
            if model.counter < -50 {
                return Some(Message::Reset);
            }
        }
        Message::Reset => model.counter = 0,
        Message::Quit => model.running_state = RunningState::Done,

        Message::List(msg) => update_list(&mut model.list, msg),
        Message::Redraw => model.draw = true,
    };
    None
}

fn update_list(model: &mut ListModel, msg: ListMessage) {
    match msg {
        ListMessage::MoveUp => {
            if model.selected > 0 {
                model.selected -= 1;
            }
        }
        ListMessage::MoveDown => {
            if model.selected < model.items.len() - 1 {
                model.selected += 1;
            }
        }
    }
}
