use chrono::{Duration, NaiveDate, NaiveTime};
use serde::{self, Deserialize, Serialize};

use crate::disp_utils::hm_from_duration;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Break {
    #[serde(rename = "break_start")]
    pub start: NaiveTime,
    #[serde(rename = "break_end")]
    pub end: NaiveTime,
}

#[derive(Debug, Serialize, Deserialize)]
struct Worked {
    start: NaiveTime,
    end: NaiveTime,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "info")]
#[serde(rename_all = "lowercase")]
pub enum DayType {
    Present {
        start: NaiveTime,
        end: NaiveTime,
        #[serde(flatten)]
        brk: Break,
    },
    HomeOffice {
        start: NaiveTime,
        end: NaiveTime,
        #[serde(flatten)]
        brk: Break,
    },
    Unofficial {
        start: NaiveTime,
        end: NaiveTime,
        #[serde(flatten)]
        brk: Option<Break>,
    },
    Travel {
        start: NaiveTime,
        end: NaiveTime,
    },
    Sick,
    Vacation,
}

impl Default for DayType {
    fn default() -> Self {
        DayType::Sick
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkDay {
    pub date: NaiveDate,
    #[serde(default)]
    #[serde(flatten)]
    pub day_type: DayType,
}

impl WorkDay {
    pub fn to_string(&self) -> String {
        match self.day_type {
            DayType::Present { start, end, .. } => {
                format!(
                    "{date} -> {:11}  {start} - {end} ({time}h)",
                    "Present",
                    start = start.format("%H:%M"),
                    end = end.format("%H:%M"),
                    date = self.date.format("%d.%m.%y"),
                    time = hm_from_duration(self.worked_time()),
                )
            }
            DayType::HomeOffice { start, end, .. } => {
                format!(
                    "{date} -> {:11}  {start} - {end} ({time}h)",
                    "Home Office",
                    start = start.format("%H:%M"),
                    end = end.format("%H:%M"),
                    date = self.date.format("%d.%m.%y"),
                    time = hm_from_duration(self.worked_time()),
                )
            }
            DayType::Unofficial { start, end, .. } => {
                format!(
                    "{date} -> {:11}  {start} - {end} ({time}h)",
                    "Unofficial",
                    start = start.format("%H:%M"),
                    end = end.format("%H:%M"),
                    date = self.date.format("%d.%m.%y"),
                    time = hm_from_duration(self.worked_time()),
                )
            }
            DayType::Sick => {
                format!("{date} -> Sick", date = self.date.format("%d.%m.%y"))
            }
            DayType::Travel { start, end } => {
                format!(
                    "{date} -> {:11}  {start} - {end} ({time}h)",
                    "Travel",
                    start = start.format("%H:%M"),
                    end = end.format("%H:%M"),
                    date = self.date.format("%d.%m.%y"),
                    time = hm_from_duration(self.worked_time()),
                )
            }
            DayType::Vacation => {
                format!("{date} -> Vacation", date = self.date.format("%d.%m.%y"))
            }
        }
    }

    pub fn worked_time(&self) -> Duration {
        match &self.day_type {
            DayType::Present { start, end, brk } => *end - *start - (brk.end - brk.start),
            DayType::HomeOffice { start, end, brk } => *end - *start - (brk.end - brk.start),
            DayType::Sick => Duration::zero(),
            DayType::Unofficial { start, end, brk } => {
                *end - *start
                    - brk
                        .as_ref()
                        .map_or(Duration::zero(), |brk| brk.end - brk.start)
            }
            DayType::Travel { start, end } => *end - *start,
            DayType::Vacation => Duration::zero(),
        }
    }

    pub fn break_time(&self) -> Duration {
        match &self.day_type {
            DayType::Present { brk, .. }
            | DayType::HomeOffice { brk, .. }
            | DayType::Unofficial { brk: Some(brk), .. } => brk.end - brk.start,
            DayType::Travel { .. }
            | DayType::Vacation
            | DayType::Sick
            | DayType::Unofficial { brk: None, .. } => Duration::zero(),
        }
    }
}
