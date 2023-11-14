use chrono::{NaiveDate, NaiveTime};
use serde::{self, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "info")]
#[serde(rename_all = "lowercase")]
pub enum DayType {
    Present {
        start: NaiveTime,
        end: NaiveTime,
        break_start: NaiveTime,
        break_end: NaiveTime,
    },
    Sick,
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
            DayType::Sick => {
                format!("{date} -> Sick", date = self.date)
            }
        }
    }

    pub fn _was_sick(&self) -> bool {
        match self.day_type {
            DayType::Present { .. } => false,
            DayType::Sick => true,
        }
    }
}
