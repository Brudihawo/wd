use clap::{Parser, Subcommand};
use serde_json;
use std::collections::HashMap;
use std::io::{stdout, Stdout};

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};

use wd::app::{handle_events, render::render_application, AppMode, AppState, Message, ORANGE};
use wd::stat::{self, StatUnit};
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
    /// Open an existing collection of work days.
    /// This is default behavior
    #[command(name = "open")]
    Open,
    /// Create a new collection of work days
    #[command(name = "create")]
    Create,
    /// Show statistics for collection
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
    action: Option<Action>,
}

fn tui_loop(mut state: AppState) -> Result<(), ()> {
    let mut terminal = init_terminal()?;

    loop {
        match terminal.draw(|frame| render_application(frame, &state)) {
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
    format!("{hours:2}:{minutes:02}")
}

fn print_stat(
    weeks: &[(chrono::NaiveDate, StatUnit)],
    total: &StatUnit,
    employ_duration: &chrono::Duration,
) -> Result<(), ()> {
    use crossterm::style::{Color, Stylize};
    let orange = match ORANGE {
        ratatui::style::Color::Rgb(r, g, b) => Color::Rgb { r, g, b },
        _ => unreachable!(),
    };

    println!(
        "{}",
        format!(
            "{:12} {:10} {:7} {:11} {:9}",
            "Week Start", "Week End", "Hours", "Active Days", "Sick Days"
        )
        .bold()
        .with(orange)
    );
    println!("{}", "=".repeat(53));
    for (week_start, stat) in weeks {
        let week_end = week_start.week(chrono::Weekday::Mon).last_day();
        println!(
            "{:12} {:10} {:7} {:11} {:9}",
            week_start.format("%d.%m.%y"),
            week_end.format("%d.%m.%y"),
            hm_from_duration(stat.work),
            stat.active_days,
            stat.sick_days,
        );
    }

    let avg_work_per_week = chrono::Duration::minutes(
        (total.work.num_minutes() as f64 / (employ_duration.num_days() as f64 / 7.0)) as i64,
    );

    println!(
        "{} {} (avg {} per week, not excluding sick days)",
        "Total Time:".bold().with(orange),
        hm_from_duration(total.work),
        hm_from_duration(avg_work_per_week)
    );

    println!("{} {}", "Sick Days:".bold().with(orange), total.sick_days);

    return Ok(());
}

fn main() -> Result<(), ()> {
    let args = Args::parse();
    match args.action {
        Some(Action::Open) | None => {
            let mut days = load_days(&args.file_path)?;
            days.sort_by_key(|day| day.date);
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
                help_popup: None,
                statistics: None,
            };
            return tui_loop(state);
        }
        Some(Action::Create) => {
            let state = AppState {
                selected: None,
                message: Message::Info(format!(
                    "Created new collection with save path {path}",
                    path = args.file_path
                )),
                file_path: args.file_path,
                days: Vec::new(),
                mode: AppMode::ListOnly,
                help_popup: None,
                statistics: None,
            };
            return tui_loop(state);
        }
        Some(Action::Stat) => {
            use crate::stat::*;
            let mut days = load_days(&args.file_path)?;
            days.sort_by_key(|day| day.date);
            if days.len() == 0 {
                eprintln!("Can not stat on empty records");
            }

            let stat_total = total_stats(&days)
                .ok_or(())
                .map_err(|()| eprintln!("Could not compute total stats"))?;
            let stat_weekly = weekly_stats(&days);
            let employ_duration = days.last().unwrap().date - days.first().unwrap().date;

            return print_stat(&stat_weekly, &stat_total, &employ_duration);
        }
    }
}
