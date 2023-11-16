use crate::app_common::colors::ORANGE;
use crate::stat::StatUnit;
use chrono::Duration;

pub fn hm_from_duration(duration: Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() - hours * 60;
    format!("{hours:02}:{minutes:02}")
}

pub fn print_stat(
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
