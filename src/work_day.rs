use chrono::{Duration, NaiveDate, NaiveTime};
use serde::{self, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "info")]
#[serde(rename_all = "lowercase")]
pub enum DayType {
    Present {
        start: NaiveTime,
        end: NaiveTime,
        #[serde(flatten)]
        brk: Break,
    },
    Sick,
    Unofficial {
        start: NaiveTime,
        end: NaiveTime,
        #[serde(flatten)]
        brk: Option<Break>,
    },
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
                format!("{date} -> Present {start} - {end}", date = self.date)
            }
            DayType::Unofficial { start, end, .. } => {
                format!("{date} -> Unofficial {start} - {end}", date = self.date)
            }
            DayType::Sick => {
                format!("{date} -> Sick", date = self.date)
            }
        }
    }

    pub fn worked_time(&self) -> Duration {
        match &self.day_type {
            DayType::Present { start, end, brk } => *end - *start - (brk.end - brk.start),
            DayType::Sick => Duration::zero(),
            DayType::Unofficial { start, end, brk } => {
                *end - *start
                    - brk
                        .as_ref()
                        .map_or(Duration::zero(), |brk| brk.end - brk.start)
            }
        }
    }
}
