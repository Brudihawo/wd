use serde_json;
use std::io::stdout;

use crossterm::{
    event,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};

use wd::app::{handle_events_edit, handle_events_listonly, render::ui, AppMode, AppState, Message};
use wd::work_day::WorkDay;

fn handle_events(state: &mut AppState) -> Result<bool, ()> {
    if event::poll(std::time::Duration::from_millis(50)).map_err(|err| {
        eprintln!("could not poll events: {err}");
    })? {
        match &state.mode {
            AppMode::ListOnly => return handle_events_listonly(state),
            AppMode::Edit { .. } => return handle_events_edit(state),
        }
    }

    Ok(false)
}

fn load_days(file_path: &str) -> Result<Vec<WorkDay>, ()> {
    let days = std::fs::read_to_string(file_path).map_err(|err| {
        eprintln!("Could not read file {file_path}: {err}");
    })?;

    serde_json::from_str(&days).map_err(|err| {
        eprintln!("Error during parsing of data: {err}");
    })
}

fn main() -> Result<(), ()> {
    let mut state = AppState {
        days: load_days("./work_times.json")?,
        selected: 0,
        mode: AppMode::ListOnly,
        message: Message::None,
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
