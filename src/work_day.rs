use chrono::{NaiveDate, NaiveTime};
use serde::{self, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Break {
    pub break_start: NaiveTime,
    pub break_end: NaiveTime,
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
}
