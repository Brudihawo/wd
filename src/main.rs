use serde_json;
use std::io::{stdout, Stdout};

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};

use wd::app::{handle_events, render::ui, AppMode, AppState, Message};
use wd::work_day::WorkDay;

fn load_days(file_path: &str) -> Result<Vec<WorkDay>, ()> {
    let days = std::fs::read_to_string(file_path).map_err(|err| {
        eprintln!("Could not read file {file_path}: {err}");
    })?;

    serde_json::from_str(&days).map_err(|err| {
        eprintln!("Error during parsing of data: {err}");
    })
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, ()> {
    let terminal = Terminal::new(CrosstermBackend::new(stdout())).map_err(|err| {
        eprintln!("Could not create terminal: {err}");
    })?;
    enable_raw_mode().map_err(|err| {
        eprintln!("Could not enable raw mode: {err}");
    })?;
    stdout()
        .execute(EnterAlternateScreen)
        .map_err(|err| eprintln!("could not enter alternate screen: {err}"))?;

    Ok(terminal)
}

fn deinit_terminal() -> Result<(), ()> {
    disable_raw_mode().map_err(|err| {
        eprintln!("Could not disable raw mode: {err}");
    })?;
    stdout()
        .execute(LeaveAlternateScreen)
        .map_err(|err| eprintln!("could not leave alternate screen: {err}"))?;
    Ok(())
}

fn main() -> Result<(), ()> {
    let mut state = AppState {
        days: load_days("./work_times.json")?,
        selected: 0,
        mode: AppMode::ListOnly,
        message: Message::None,
    };
    let mut terminal = init_terminal()?;

    loop {
        match terminal.draw(|frame| ui(frame, &state)) {
            Ok(_) => match handle_events(&mut state) {
                Ok(false) => (),
                Ok(true) | Err(()) => break,
            },
            Err(err) => {
                eprintln!("Could not draw UI: {err}");
                break;
            }
        };
    }

    deinit_terminal()?;
    Ok(())
}
