use crate::disp_utils::hm_from_duration;
use crate::editor::{EditDayType, EditField, EditMode};
use ratatui::{
    prelude::{Color, Frame, Rect, Style, Stylize},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

pub use crate::app_common::*;

pub use crate::app_common::colors::*;
pub use crate::app_common::SCROLL_AMT;

pub mod render;
pub mod events;
