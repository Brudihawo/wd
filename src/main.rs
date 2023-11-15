use chrono::{Duration, NaiveTime, NaiveWeek};
use clap::{Parser, Subcommand};
use serde_json;
use std::collections::HashMap;
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

struct StatUnit {
    work: chrono::Duration,
    active_days: u8,

    brk: chrono::Duration,

    mean_start: chrono::NaiveTime,
    mean_end: chrono::NaiveTime,
}

fn dur_since_mn(time: chrono::NaiveTime) -> chrono::Duration {
    time - chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()
}

impl StatUnit {
    pub fn from_single_day(day: &WorkDay) -> Option<Self> {
        use wd::work_day::DayType;
        match &day.day_type {
            DayType::Present { start, end, .. } | DayType::Unofficial { start, end, .. } => {
                Some(Self {
                    work: day.worked_time(),
                    active_days: 1,
                    brk: day.break_time(),
                    mean_start: *start,
                    mean_end: *end,
                })
            }
            DayType::Sick => None,
        }
    }

    pub fn update_mean_start(&mut self, start: &chrono::NaiveTime) {
        let ms_since_mn = dur_since_mn(self.mean_start) * self.active_days as i32;
        let cs_since_mn = dur_since_mn(*start);

        let new_avg_duration = Duration::minutes(
            (ms_since_mn + cs_since_mn).num_minutes() / (self.active_days + 1) as i64,
        );
        let hours = new_avg_duration.num_hours();
        let minutes = new_avg_duration.num_minutes() - hours * 60;

        self.mean_start = NaiveTime::from_hms_opt(hours as u32, minutes as u32, 0).unwrap();
    }

    pub fn update_mean_end(&mut self, end: &chrono::NaiveTime) {
        let me_since_mn = dur_since_mn(self.mean_end) * self.active_days as i32;
        let ce_since_mn = dur_since_mn(*end);

        let new_avg_duration = Duration::minutes(
            (me_since_mn + ce_since_mn).num_minutes() / (self.active_days + 1) as i64,
        );
        let hours = new_avg_duration.num_hours();
        let minutes = new_avg_duration.num_minutes() - hours * 60;

        self.mean_end = NaiveTime::from_hms_opt(hours as u32, minutes as u32, 0).unwrap();
    }

    pub fn push_day(&mut self, day: &WorkDay) {
        use wd::work_day::DayType;
        match &day.day_type {
            DayType::Present { start, end, .. } | DayType::Unofficial { start, end, .. } => {
                self.update_mean_start(start);
                self.update_mean_end(end);
                self.brk = self.brk + day.break_time();
                self.work = self.work + day.worked_time();

                self.active_days += 1;
            }
            DayType::Sick => (),
        }
    }
}

impl Default for StatUnit {
    fn default() -> Self {
        Self {
            work: chrono::Duration::zero(),
            active_days: 0,
            brk: chrono::Duration::zero(),
            mean_start: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            mean_end: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        }
    }
}

fn print_stat(
    weeks: &[(&chrono::NaiveDate, &StatUnit)],
    total: &StatUnit,
    employ_duration: &chrono::Duration,
) -> Result<(), ()> {
    use crossterm::style::{Color, Stylize};
    let orange = match ORANGE {
        ratatui::style::Color::Rgb(r, g, b) => Color::Rgb { r, g, b },
        _ => unreachable!(),
    };

    for (week_start, stat) in weeks {
        let week_end = week_start.week(chrono::Weekday::Mon).last_day();
        println!(
            "Week {} to {} {}",
            week_start.format("%d.%m.%y"),
            week_end.format("%d.%m.%y"),
            hm_from_duration(stat.work),
        );
    }

    let avg_per_week = chrono::Duration::minutes(
        (total.work.num_minutes() as f64 / (employ_duration.num_days() as f64 / 7.0)) as i64,
    );

    println!(
        "{} {} (avg {} per week, not excluding sick days)\n",
        "Total Time:".bold().with(orange),
        hm_from_duration(total.work),
        hm_from_duration(avg_per_week)
    );

    return Ok(());
}

fn hm_from_duration(duration: chrono::Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() - hours * 60;
    format!("{hours}h {minutes}m")
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
            };
            return tui_loop(state);
        }
        Some(Action::Stat) => {
            let mut days = load_days(&args.file_path)?;
            days.sort_by_key(|day| day.date);
            if days.len() == 0 {
                eprintln!("Can not stat on empty records");
            }

            let mut stat_weeks: HashMap<chrono::NaiveDate, StatUnit> = HashMap::new();
            let mut stat_total: Option<StatUnit> = None;

            for day in days.iter() {
                if let Some(total) = stat_total.as_mut() {
                    total.push_day(day);
                } else {
                    stat_total = StatUnit::from_single_day(day);
                }

                if let Some(ws) = StatUnit::from_single_day(day) {
                    stat_weeks
                        .entry(day.date.week(chrono::Weekday::Mon).first_day())
                        .and_modify(|entry| entry.push_day(day))
                        .or_insert(ws);
                }
            }
            let mut weeks = stat_weeks.iter().collect::<Vec<_>>();
            let employ_duration = days.last().unwrap().date - days.first().unwrap().date;

            weeks.sort_by_key(|(week_start, _)| *week_start);

            return print_stat(&weeks, &stat_total.unwrap(), &employ_duration);
        }
    }
}
