use crate::editor::{EditBufs, EditField, EditMode};
use crate::work_day::WorkDay;
use chrono::NaiveDate;
use crate::stat::StatUnit;

pub mod colors {
    use ratatui::prelude::Color;
    pub const ORANGE: Color = Color::Rgb(255, 140, 0);

    pub const MOVE_CLR: Color = ORANGE;
    pub const EDIT_MOVE_CLR: Color = Color::LightCyan;
    pub const EDIT_INS_CLR: Color = Color::LightYellow;
    pub const HELP_CLR: Color = Color::LightGreen;
    pub const STAT_CLR: Color = Color::LightMagenta;
}

pub const SCROLL_AMT: usize = 5;

pub enum AppMode {
    ListOnly,
    Edit {
        mode: EditMode,
        field: EditField,
        edit_bufs: EditBufs,
        index: usize,
    },
}

pub enum Message {
    Info(String),
    Error(String),
    None,
}

pub struct StatsState {
    pub weekly: Vec<(NaiveDate, StatUnit)>,
    pub total: StatUnit,
    pub scroll: usize,
    pub week_hours: f32,
}

pub struct Settings {
    pub week_hours: f32,
}

pub struct AppState {
    pub file_path: String,
    pub days: Vec<WorkDay>,
    pub settings: Settings,
    pub selected: Option<usize>,
    pub mode: AppMode,
    pub message: Message,
    pub help_popup: Option<usize>,
    pub statistics: Option<StatsState>,
}

impl AppState {
    pub fn write(&mut self) -> Result<(), ()> {
        use std::fs::File;
        use std::io::BufWriter;

        let writer = BufWriter::new(File::create(&self.file_path).map_err(|err| {
            self.message = Message::Error(format!("Could not open file {}: {err}", self.file_path))
        })?);
        let w_res = serde_json::to_writer_pretty(writer, &self.days);
        match w_res {
            Ok(()) => (),
            Err(err) => {
                self.message =
                    Message::Error(format!("Could not open file {}: {err}", &self.file_path))
            }
        }
        self.message = Message::Info(format!(
            "Wrote {} entries to file {}",
            self.days.len(),
            &self.file_path
        ));

        Ok(())
    }

    pub fn next_day(&mut self) -> Option<usize> {
        if let Some(selected) = self.selected {
            if selected < self.days.len() - 1 {
                Some(selected + 1)
            } else {
                Some(selected)
            }
        } else {
            None
        }
    }

    pub fn prev_day(&self) -> Option<usize> {
        if let Some(selected) = self.selected {
            if selected > 0 {
                Some(selected - 1)
            } else {
                Some(selected)
            }
        } else {
            None
        }
    }
}
