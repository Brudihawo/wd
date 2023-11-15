use crate::work_day::{Break, DayType, WorkDay};
use chrono::{NaiveDate, NaiveTime};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EditField {
    Date,
    Start,
    End,
    BreakStart,
    BreakEnd,
    DayType,
}

#[derive(Copy, Clone)]
pub enum EditDayType {
    Present,
    Sick,
    Unofficial { has_break: bool },
}

impl EditDayType {
    pub fn next(&self) -> Self {
        use EditDayType::*;
        match self {
            Present => Sick,
            Sick => Unofficial { has_break: false },
            Unofficial { has_break: false } => Unofficial { has_break: true },
            Unofficial { has_break: true } => Present,
        }
    }

    pub fn prev(&self) -> Self {
        use EditDayType::*;
        match self {
            Present => Unofficial { has_break: true },
            Sick => Present,
            Unofficial { has_break: false } => Sick,
            Unofficial { has_break: true } => Unofficial { has_break: false },
        }
    }
}

impl EditField {
    pub fn next(&self, day_type: EditDayType) -> Self {
        use EditField::*;
        match day_type {
            EditDayType::Present => match self {
                Date => DayType,
                DayType => Start,
                Start => End,
                End => BreakStart,
                BreakStart => BreakEnd,
                BreakEnd => Date,
            },
            EditDayType::Sick => match self {
                Date => DayType,
                DayType => Date,
                Start | End | BreakStart | BreakEnd => unreachable!(),
            },
            EditDayType::Unofficial { has_break } => {
                if has_break {
                    match self {
                        Date => DayType,
                        DayType => Start,
                        Start => End,
                        End => BreakStart,
                        BreakStart => BreakEnd,
                        BreakEnd => Date,
                    }
                } else {
                    match self {
                        Date => DayType,
                        DayType => Start,
                        Start => End,
                        End => Date,
                        BreakStart | BreakEnd => unreachable!(),
                    }
                }
            }
        }
    }

    pub fn prev(&self, day_type: EditDayType) -> Self {
        use EditField::*;
        match day_type {
            EditDayType::Present => match self {
                Date => BreakEnd,
                DayType => Date,
                Start => DayType,
                End => Start,
                BreakStart => End,
                BreakEnd => BreakStart,
            },
            EditDayType::Sick => match self {
                Date => DayType,
                DayType => Date,
                Start | End | BreakStart | BreakEnd => unreachable!(),
            },
            EditDayType::Unofficial { has_break } => {
                if has_break {
                    match self {
                        Date => BreakEnd,
                        DayType => Date,
                        Start => DayType,
                        End => Start,
                        BreakStart => End,
                        BreakEnd => BreakStart,
                    }
                } else {
                    match self {
                        Date => End,
                        DayType => Date,
                        Start => DayType,
                        End => Start,
                        BreakStart | BreakEnd => unreachable!(),
                    }
                }
            }
        }
    }
}

pub struct EditBufs {
    bufs: [[u8; 64]; 5],
    cursors: [u8; 5],
    pub day_type: EditDayType,
}

impl EditBufs {
    fn new() -> Self {
        Self {
            bufs: [[0; 64]; 5],
            cursors: [0; 5],
            day_type: EditDayType::Present,
        }
    }

    pub fn entry_mut(&mut self, index: EditField) -> (&mut [u8; 64], &mut u8) {
        match index {
            EditField::Date => (&mut self.bufs[0], &mut self.cursors[0]),
            EditField::Start => (&mut self.bufs[1], &mut self.cursors[1]),
            EditField::End => (&mut self.bufs[2], &mut self.cursors[2]),
            EditField::BreakStart => (&mut self.bufs[3], &mut self.cursors[3]),
            EditField::BreakEnd => (&mut self.bufs[4], &mut self.cursors[4]),
            EditField::DayType => unreachable!(),
        }
    }

    pub fn entry(&self, index: EditField) -> (&[u8; 64], &u8) {
        match index {
            EditField::Date => (&self.bufs[0], &self.cursors[0]),
            EditField::Start => (&self.bufs[1], &self.cursors[1]),
            EditField::End => (&self.bufs[2], &self.cursors[2]),
            EditField::BreakStart => (&self.bufs[3], &self.cursors[3]),
            EditField::BreakEnd => (&self.bufs[4], &self.cursors[4]),
            EditField::DayType => unreachable!(),
        }
    }

    pub fn text(&self, index: EditField) -> &str {
        let (buf, cur) = self.entry(index);
        std::str::from_utf8(&buf[..*cur as usize]).unwrap()
    }
}

impl From<&WorkDay> for EditBufs {
    fn from(day: &WorkDay) -> Self {
        use EditField as E;
        let mut ret = Self::new();
        ret.cursors[E::Date as usize] = ret[E::Date]
            .iter_mut()
            .zip(day.date.to_string().as_bytes())
            .map(|(b, sb)| *b = *sb)
            .count() as u8;
        match &day.day_type {
            DayType::Present {
                start,
                end,
                brk:
                    Break {
                        start: break_start,
                        end: break_end,
                    },
            } => {
                ret.cursors[E::Start as usize] = ret[E::Start]
                    .iter_mut()
                    .zip(start.to_string().as_bytes())
                    .map(|(b, sb)| *b = *sb)
                    .count() as u8;
                ret.cursors[E::End as usize] = ret[E::End]
                    .iter_mut()
                    .zip(end.to_string().as_bytes())
                    .map(|(b, sb)| *b = *sb)
                    .count() as u8;
                ret.cursors[E::BreakStart as usize] = ret[E::BreakStart]
                    .iter_mut()
                    .zip(break_start.to_string().as_bytes())
                    .map(|(b, sb)| *b = *sb)
                    .count() as u8;
                ret.cursors[E::BreakEnd as usize] = ret[E::BreakEnd]
                    .iter_mut()
                    .zip(break_end.to_string().as_bytes())
                    .map(|(b, sb)| *b = *sb)
                    .count() as u8;
                ret.day_type = EditDayType::Present;
            }
            DayType::Sick => ret.day_type = EditDayType::Sick,
            DayType::Unofficial { start, end, brk } => {
                ret.cursors[E::Start as usize] = ret[E::Start]
                    .iter_mut()
                    .zip(start.to_string().as_bytes())
                    .map(|(b, sb)| *b = *sb)
                    .count() as u8;
                ret.cursors[E::End as usize] = ret[E::End]
                    .iter_mut()
                    .zip(end.to_string().as_bytes())
                    .map(|(b, sb)| *b = *sb)
                    .count() as u8;
                if let Some(b) = brk {
                    ret.cursors[E::BreakStart as usize] = ret[E::BreakStart]
                        .iter_mut()
                        .zip(b.start.to_string().as_bytes())
                        .map(|(b, sb)| *b = *sb)
                        .count() as u8;
                    ret.cursors[E::BreakEnd as usize] = ret[E::BreakEnd]
                        .iter_mut()
                        .zip(b.end.to_string().as_bytes())
                        .map(|(b, sb)| *b = *sb)
                        .count() as u8;
                }
                ret.day_type = EditDayType::Sick;
            }
        }
        ret
    }
}

impl TryInto<WorkDay> for &EditBufs {
    type Error = String;

    fn try_into(self) -> Result<WorkDay, Self::Error> {
        Ok(WorkDay {
            date: NaiveDate::parse_from_str(self.text(EditField::Date), "%Y-%m-%d")
                .map_err(|err| format!("could not parse Date: {err}"))?,
            day_type: match self.day_type {
                EditDayType::Present => DayType::Present {
                    start: NaiveTime::parse_from_str(self.text(EditField::Start), "%H:%M:%S")
                        .map_err(|err| format!("could not parse Start: {err}"))?,
                    end: NaiveTime::parse_from_str(self.text(EditField::End), "%H:%M:%S")
                        .map_err(|err| format!("could not parse End: {err}"))?,
                    brk: Break {
                        start: NaiveTime::parse_from_str(
                            self.text(EditField::BreakStart),
                            "%H:%M:%S",
                        )
                        .map_err(|err| format!("could not parse Break Start: {err}"))?,
                        end: NaiveTime::parse_from_str(
                            self.text(EditField::BreakEnd),
                            "%H:%M:%S",
                        )
                        .map_err(|err| format!("could not parse Break End {err}"))?,
                    },
                },
                EditDayType::Sick => DayType::Sick,
                EditDayType::Unofficial { has_break } => DayType::Unofficial {
                    start: NaiveTime::parse_from_str(self.text(EditField::Start), "%H:%M:%S")
                        .map_err(|err| format!("could not parse Start: {err}"))?,
                    end: NaiveTime::parse_from_str(self.text(EditField::End), "%H:%M:%S")
                        .map_err(|err| format!("could not parse End: {err}"))?,
                    brk: if has_break {
                        Some(Break {
                            start: NaiveTime::parse_from_str(
                                self.text(EditField::BreakStart),
                                "%H:%M:%S",
                            )
                            .map_err(|err| format!("could not parse Break Start: {err}"))?,
                            end: NaiveTime::parse_from_str(
                                self.text(EditField::BreakEnd),
                                "%H:%M:%S",
                            )
                            .map_err(|err| format!("could not parse Break End {err}"))?,
                        })
                    } else {
                        None
                    },
                },
            },
        })
    }
}

impl std::ops::Index<EditField> for EditBufs {
    type Output = [u8; 64];

    fn index(&self, index: EditField) -> &Self::Output {
        match index {
            EditField::Date => &self.bufs[0],
            EditField::Start => &self.bufs[1],
            EditField::End => &self.bufs[2],
            EditField::BreakStart => &self.bufs[3],
            EditField::BreakEnd => &self.bufs[4],
            EditField::DayType => unreachable!(),
        }
    }
}

impl std::ops::IndexMut<EditField> for EditBufs {
    fn index_mut(&mut self, index: EditField) -> &mut Self::Output {
        match index {
            EditField::Date => &mut self.bufs[0],
            EditField::Start => &mut self.bufs[1],
            EditField::End => &mut self.bufs[2],
            EditField::BreakStart => &mut self.bufs[3],
            EditField::BreakEnd => &mut self.bufs[4],
            EditField::DayType => unreachable!(),
        }
    }
}

pub enum EditMode {
    Move,
    Insert,
}
