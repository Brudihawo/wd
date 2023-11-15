use clap::{Parser, Subcommand};
use serde_json;
use std::io::{stdout, Stdout};

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};

use wd::app::{handle_events, render::ui, AppMode, AppState, Message, ORANGE};
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

#[derive(Subcommand)]
#[command(about = "test")]
enum Action {
    /// Open an existing collection of work days
    #[command(name = "open")]
    Open,
    /// Create a new collection of work days
    #[command(name = "create")]
    Create,
    /// Show statistics for file
    #[command(name = "stat")]
    Stat,
}

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Keep track of your work times in the terminal",
    long_about = None
)]
struct Args {
    #[arg(default_value = "work_times.json")]
    file_path: String,
    #[command(subcommand)]
    action: Action,
}

fn tui_loop(mut state: AppState) -> Result<(), ()> {
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

fn hm_from_duration(duration: chrono::Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() - hours * 60;
    format!("{hours}h {minutes}m")
}

fn main() -> Result<(), ()> {
    let args = Args::parse();
    match args.action {
        Action::Open => {
            let days = load_days(&args.file_path)?;
            let state = AppState {
                selected: Some(days.len() - 1),
                message: Message::Info(format!(
                    "Loaded {len} entries from {path}",
                    len = days.len(),
                    path = args.file_path
                )),
                file_path: args.file_path,
                days,
                mode: AppMode::ListOnly,
            };
            return tui_loop(state);
        }
        Action::Create => {
            let state = AppState {
                selected: None,
                message: Message::Info(format!(
                    "Created new collection with save path {path}",
                    path = args.file_path
                )),
                file_path: args.file_path,
                days: Vec::new(),
                mode: AppMode::ListOnly,
            };
            return tui_loop(state);
        }
        Action::Stat => {
            use crossterm::style::{Color, Stylize};
            let orange = match ORANGE {
                ratatui::style::Color::Rgb(r, g, b) => Color::Rgb { r, g, b },
                _ => unreachable!(),
            };
            let mut days = load_days(&args.file_path)?;

            if days.len() == 0 {
                eprintln!("Can not stat on empty records");
            }

            days.sort_by_key(|day| day.date);
            let employ_duration = days.last().unwrap().date - days.first().unwrap().date;

            let total_work_duration: chrono::Duration =
                days.iter().map(|day| day.worked_time()).sum();
            let avg_per_week = chrono::Duration::minutes(
                (total_work_duration.num_minutes() as f64
                    / (employ_duration.num_days() as f64 / 7.0)) as i64,
            );

            println!(
                "{} {} (avg {} per week, not excluding sick days)\n",
                "Total Time:".bold().with(orange),
                hm_from_duration(total_work_duration),
                hm_from_duration(avg_per_week)
            );

            let today = chrono::Local::now().naive_local().date();
            let this_week = today.week(chrono::Weekday::Mon);
            let this_week_start = this_week.first_day();

            let this_week_work: chrono::Duration = days
                .iter()
                .filter(|day| day.date.week(chrono::Weekday::Mon).first_day() == this_week_start)
                .map(|day| day.worked_time())
                .sum();

            println!(
                "{} {} ({} to {})",
                format!("Time This Week:",).bold().with(orange),
                hm_from_duration(this_week_work),
                this_week_start.format("%d.%m.%y"),
                this_week.last_day().format("%d.%m.%y")
            );

            return Ok(());
        }
    }
}
