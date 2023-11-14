use chrono::{Local, NaiveTime};
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{self, stdout};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "info")]
#[serde(rename_all = "lowercase")]
enum DayType {
    Present {
        start: chrono::NaiveTime,
        end: chrono::NaiveTime,
        break_start: chrono::NaiveTime,
        break_end: chrono::NaiveTime,
    },
    Sick,
}

impl Default for DayType {
    fn default() -> Self {
        DayType::Sick
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkDay {
    date: chrono::NaiveDate,
    #[serde(default)]
    #[serde(flatten)]
    day_type: DayType,
}

impl WorkDay {
    fn to_string(&self) -> String {
        match self.day_type {
            DayType::Present {
                start,
                end,
                break_start,
                break_end,
            } => {
                format!("{date} -> Present {start} - {end}", date = self.date)
            }
            DayType::Sick => {
                format!("{date} -> Sick", date = self.date)
            }
        }
    }
}

enum WorkDayField {
    Date,
    Start,
    End,
    BreakStart,
    BreakEnd,
}

enum AppMode {
    ListOnly,
    Edit(usize, WorkDayField),
}

struct AppState {
    days: Vec<WorkDay>,
    selected: usize,
    mode: AppMode,
}

fn handle_events_edit(state: &mut AppState) -> Result<bool, ()> {
    if let Event::Key(key) = event::read().map_err(|err| {
        eprintln!("Could not read event: {err}");
    })? {
        if key.kind == event::KeyEventKind::Press {
            match key.code {
                KeyCode::Esc => state.mode = AppMode::ListOnly,
                _ => (),
            }
        }
    }

    Ok(false)
}

fn handle_events_listonly(state: &mut AppState) -> Result<bool, ()> {
    if let Event::Key(key) = event::read().map_err(|err| {
        eprintln!("Could not read event: {err}");
    })? {
        if key.kind == event::KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('j') => {
                    if state.selected < state.days.len() - 1 {
                        state.selected += 1;
                    }
                }
                KeyCode::Char('k') => {
                    if state.selected >= 1 {
                        state.selected -= 1;
                    }
                }
                KeyCode::Char('e') => {}
                KeyCode::Char('+') => {}
                _ => (),
            }
        }
    }

    Ok(false)
}

fn handle_events(state: &mut AppState) -> Result<bool, ()> {
    if event::poll(std::time::Duration::from_millis(50)).map_err(|err| {
        eprintln!("could not poll events: {err}");
    })? {
        match state.mode {
            AppMode::ListOnly => return handle_events_listonly(state),
            AppMode::Edit(..) => return handle_events_edit(state),
        }
    }

    Ok(false)
}

fn ui(frame: &mut Frame, state: &AppState) {
    frame.render_stateful_widget(
        List::new(
            state
                .days
                .iter()
                .map(|day| ListItem::new(day.to_string()))
                .collect::<Vec<_>>(),
        )
        .highlight_symbol(">>")
        .highlight_style(Style::default().fg(Color::Rgb(255, 140, 0)).bold()),
        frame.size(),
        &mut ListState::default().with_selected(Some(state.selected)),
    );
    if let AppMode::Edit(index, field) = &state.mode {
        let day = &state.days[*index];
        frame.render_widget(
            List::new([ListItem::new(format!("Date: {}", day.date))]),
            frame.size(),
        );
        // match day.day_type {
        //     DayType::Present{
        //         start,
        //         end,
        //         break_start,
        //         break_end,
        //     } => {
        //         format!()
        //     }
        // }
    }
}

fn main() -> Result<(), ()> {
    let file_name = "./work_times.json";
    let days = std::fs::read_to_string(file_name).map_err(|err| {
        eprintln!("Could not read file {file_name}: {err}");
    })?;

    let days: Vec<WorkDay> = serde_json::from_str(&days).map_err(|err| {
        eprintln!("Error during parsing of data: {err}");
        eprintln!("data: ")
    })?;

    let mut state = AppState {
        days,
        selected: 0,
        mode: AppMode::ListOnly,
    };

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).map_err(|err| {
        eprintln!("Could not create terminal: {err}");
    })?;
    enable_raw_mode().map_err(|err| {
        eprintln!("Could not enable raw mode: {err}");
    })?;
    stdout()
        .execute(EnterAlternateScreen)
        .map_err(|err| eprintln!("could not enter alternate screen: {err}"))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|frame| ui(frame, &state)).map_err(|err| {
            eprintln!("Could not draw UI: {err}");
            should_quit = true;
        })?;
        should_quit = handle_events(&mut state)?;
    }

    disable_raw_mode().map_err(|err| {
        eprintln!("Could not disable raw mode: {err}");
    })?;
    stdout()
        .execute(LeaveAlternateScreen)
        .map_err(|err| eprintln!("could not leave alternate screen: {err}"))?;
    Ok(())
}
