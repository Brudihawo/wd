use crate::work_day::{DayType, WorkDay};
use chrono::{Duration, NaiveDate, NaiveTime};
use std::collections::HashMap;

pub struct StatUnit {
    pub work: Duration,
    pub active_days: u32,
    pub sick_days: u32,

    pub brk: Duration,

    pub mean_start: Option<NaiveTime>,
    pub mean_end: Option<NaiveTime>,
}

impl StatUnit {
    pub fn from_single_day(day: &WorkDay) -> Self {
        match &day.day_type {
            DayType::Present { start, end, .. } | DayType::Unofficial { start, end, .. } => Self {
                work: day.worked_time(),
                active_days: 1,
                sick_days: 0,
                brk: day.break_time(),
                mean_start: Some(*start),
                mean_end: Some(*end),
            },
            DayType::Sick => {
                let mut ret = Self::default();
                ret.sick_days = 1;
                return ret;
            }
        }
    }

    pub fn update_mean_start(&mut self, start: &NaiveTime) {
        let ms_since_mn =
            self.mean_start.map_or(Duration::zero(), dur_since_mn) * self.active_days as i32;
        let cs_since_mn = dur_since_mn(*start);

        let new_avg_duration = Duration::minutes(
            (ms_since_mn + cs_since_mn).num_minutes() / (self.active_days + 1) as i64,
        );
        let hours = new_avg_duration.num_hours();
        let minutes = new_avg_duration.num_minutes() - hours * 60;

        self.mean_start = Some(NaiveTime::from_hms_opt(hours as u32, minutes as u32, 0).unwrap())
    }

    pub fn update_mean_end(&mut self, end: &NaiveTime) {
        let me_since_mn =
            self.mean_end.map_or(Duration::zero(), dur_since_mn) * self.active_days as i32;
        let ce_since_mn = dur_since_mn(*end);

        let new_avg_duration = Duration::minutes(
            (me_since_mn + ce_since_mn).num_minutes() / (self.active_days + 1) as i64,
        );
        let hours = new_avg_duration.num_hours();
        let minutes = new_avg_duration.num_minutes() - hours * 60;

        self.mean_end = Some(NaiveTime::from_hms_opt(hours as u32, minutes as u32, 0).unwrap());
    }

    pub fn push_day(&mut self, day: &WorkDay) {
        match &day.day_type {
            DayType::Present { start, end, .. } | DayType::Unofficial { start, end, .. } => {
                self.update_mean_start(start);
                self.update_mean_end(end);
                self.brk = self.brk + day.break_time();
                self.work = self.work + day.worked_time();

                self.active_days += 1;
            }
            DayType::Sick => self.sick_days += 1,
        }
    }
}

pub fn total_stats(days: &[WorkDay]) -> Option<StatUnit> {
    let mut total = StatUnit::from_single_day(days.first()?);

    for day in days.iter().skip(1) {
        total.push_day(day);
    }

    Some(total)
}

pub fn weekly_stats(days: &[WorkDay]) -> Vec<(NaiveDate, StatUnit)> {
    let mut stat_weeks: HashMap<NaiveDate, StatUnit> = HashMap::new();

    for day in days.iter() {
        stat_weeks
            .entry(day.date.week(chrono::Weekday::Mon).first_day())
            .and_modify(|entry| entry.push_day(day))
            .or_insert(StatUnit::from_single_day(day));
    }
    let mut weeks = stat_weeks.drain().collect::<Vec<_>>();
    weeks.sort_by_key(|(week_start, _)| *week_start);
    weeks
}

impl Default for StatUnit {
    fn default() -> Self {
        Self {
            work: Duration::zero(),
            active_days: 0,
            sick_days: 0,
            brk: Duration::zero(),
            mean_start: None,
            mean_end: None,
        }
    }
}

fn dur_since_mn(time: NaiveTime) -> Duration {
    time - NaiveTime::from_hms_opt(0, 0, 0).unwrap()
}
