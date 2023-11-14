use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::stdout;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};

const ORANGE: Color = Color::Rgb(255, 140, 0);

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "info")]
#[serde(rename_all = "lowercase")]
enum DayType {
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
struct WorkDay {
    date: NaiveDate,
    #[serde(default)]
    #[serde(flatten)]
    day_type: DayType,
}

impl WorkDay {
    fn to_string(&self) -> String {
        match self.day_type {
            DayType::Present { start, end, .. } => {
                format!("{date} -> Present {start} - {end}", date = self.date)
            }
            DayType::Sick => {
                format!("{date} -> Sick", date = self.date)
            }
        }
    }

    fn _was_sick(&self) -> bool {
        match self.day_type {
            DayType::Present { .. } => false,
            DayType::Sick => true,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum EditField {
    Date,
    Sick,
    Start,
    End,
    BreakStart,
    BreakEnd,
}

impl EditField {
    fn next(&self, sick: bool) -> Self {
        use EditField::*;
        if !sick {
            match self {
                Date => Sick,
                Sick => Start,
                Start => End,
                End => BreakStart,
                BreakStart => BreakEnd,
                BreakEnd => Date,
            }
        } else {
            match self {
                Date => Sick,
                Sick => Date,
                Start | End | BreakStart | BreakEnd => unreachable!(),
            }
        }
    }

    fn prev(&self, sick: bool) -> Self {
        use EditField::*;
        if !sick {
            match self {
                Date => BreakEnd,
                Sick => Date,
                Start => Sick,
                End => Start,
                BreakStart => End,
                BreakEnd => BreakStart,
            }
        } else {
            match self {
                Date => Sick,
                Sick => Date,
                Start | End | BreakStart | BreakEnd => unreachable!(),
            }
        }
    }
}

struct EditBufs {
    bufs: [[u8; 64]; 5],
    cursors: [u8; 5],
    sick: bool,
}

impl EditBufs {
    fn new() -> Self {
        Self {
            bufs: [[0; 64]; 5],
            cursors: [0; 5],
            sick: false,
        }
    }

    fn from_day(day: &WorkDay) -> Self {
        use EditField as E;
        let mut ret = Self::new();
        ret[E::Date]
            .iter_mut()
            .zip(day.date.to_string().as_bytes())
            .map(|(b, sb)| *b = *sb)
            .count();
        match day.day_type {
            DayType::Present {
                start,
                end,
                break_start,
                break_end,
            } => {
                ret[E::Start]
                    .iter_mut()
                    .zip(start.to_string().as_bytes())
                    .map(|(b, sb)| *b = *sb)
                    .count();
                ret[E::End]
                    .iter_mut()
                    .zip(end.to_string().as_bytes())
                    .map(|(b, sb)| *b = *sb)
                    .count();
                ret[E::BreakStart]
                    .iter_mut()
                    .zip(break_start.to_string().as_bytes())
                    .map(|(b, sb)| *b = *sb)
                    .count();
                ret[E::BreakEnd]
                    .iter_mut()
                    .zip(break_end.to_string().as_bytes())
                    .map(|(b, sb)| *b = *sb)
                    .count();
            }
            DayType::Sick => ret.sick = false,
        }
        ret
    }

    fn entry(&mut self, index: EditField) -> (&mut [u8; 64], &mut u8) {
        match index {
            EditField::Date => (&mut self.bufs[0], &mut self.cursors[0]),
            EditField::Start => (&mut self.bufs[1], &mut self.cursors[1]),
            EditField::End => (&mut self.bufs[2], &mut self.cursors[2]),
            EditField::BreakStart => (&mut self.bufs[3], &mut self.cursors[3]),
            EditField::BreakEnd => (&mut self.bufs[4], &mut self.cursors[4]),
            EditField::Sick => unreachable!(),
        }
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
            EditField::Sick => unreachable!(),
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
            EditField::Sick => unreachable!(),
        }
    }
}

enum EditMode {
    Move,
    Edit,
}

enum AppMode {
    ListOnly,
    Edit {
        mode: EditMode,
        field: EditField,
        edit_bufs: EditBufs,
    },
}

struct AppState {
    days: Vec<WorkDay>,
    selected: usize,
    mode: AppMode,
}

fn render_edit_window(frame: &mut Frame, pos: &Rect, state: &AppState) {
    if let AppMode::Edit {
        field,
        edit_bufs,
        mode: e_mode,
        ..
    } = &state.mode
    {
        let mut list = vec![
            ListItem::new("Date"),
            ListItem::new(if edit_bufs.sick { "Sick" } else { "Present" }),
        ];

        if !edit_bufs.sick {
            list.extend([
                ListItem::new("Start"),
                ListItem::new("End"),
                ListItem::new("Break Start"),
                ListItem::new("Break End"),
            ])
        }

        frame.render_stateful_widget(
            List::new(list)
                .block(Block::default().title("Edit Date").borders(Borders::ALL))
                .highlight_style(Style::default().bold().fg(match e_mode {
                    EditMode::Move => ORANGE,
                    EditMode::Edit => Color::LightBlue,
                })),
            *pos,
            &mut ListState::default().with_selected(Some(*field as usize)),
        );
    } else {
        unreachable!()
    }
}

fn handle_events_edit(state: &mut AppState) -> Result<bool, ()> {
    if let Event::Key(key) = event::read().map_err(|err| {
        eprintln!("Could not read event: {err}");
    })? {
        if key.kind == event::KeyEventKind::Press {
            let (edit_bufs, field, e_mode) = match &mut state.mode {
                AppMode::ListOnly => unreachable!(),
                AppMode::Edit {
                    edit_bufs,
                    field,
                    mode,
                    ..
                } => (edit_bufs, field, mode),
            };

            match e_mode {
                EditMode::Move => match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Char('j') => *field = field.next(edit_bufs.sick),
                    KeyCode::Char('k') => *field = field.prev(edit_bufs.sick),
                    KeyCode::Esc | KeyCode::Char('h') => state.mode = AppMode::ListOnly,
                    KeyCode::Enter | KeyCode::Char('l') => {
                        if *field == EditField::Sick {
                            edit_bufs.sick = !edit_bufs.sick;
                        } else {
                            *e_mode = EditMode::Edit;
                        }
                    }
                    _ => (),
                },
                EditMode::Edit => match key.code {
                    KeyCode::Esc => {
                        *e_mode = EditMode::Move;
                    }
                    KeyCode::Char(c) => {
                        assert_ne!(*field, EditField::Sick);
                        if c.is_ascii() {
                            let (buf, cur) = edit_bufs.entry(*field);
                            if *cur < 64 {
                                buf[*cur as usize] = c as u8;
                                *cur += 1;
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        assert_ne!(*field, EditField::Sick);
                        let (_, cur) = edit_bufs.entry(*field);
                        if *cur < 1 {
                            *cur -= 1;
                        }
                    }
                    _ => (),
                },
            }
        }
    }

    Ok(false)
}

fn render_list(frame: &mut Frame, pos: &Rect, state: &AppState, active: bool) {
    frame.render_stateful_widget(
        List::new(
            state
                .days
                .iter()
                .map(|day| ListItem::new(day.to_string()))
                .collect::<Vec<_>>(),
        )
        .block(
            Block::default()
                .title("Work Days")
                .borders(Borders::ALL)
                .border_style(if active {
                    Style::default().fg(ORANGE)
                } else {
                    Style::default().fg(Color::Gray)
                }),
        )
        .highlight_symbol(">>")
        .highlight_style(Style::default().fg(ORANGE).bold()),
        *pos,
        &mut ListState::default().with_selected(Some(state.selected)),
    );
}

fn handle_events_listonly(state: &mut AppState) -> Result<bool, ()> {
    if let Event::Key(key) = event::read().map_err(|err| {
        eprintln!("Could not read event: {err}");
    })? {
        if key.kind == event::KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('j') => {
                    if state.selected < state.days.len() - 1 {
                        state.selected += 1;
                    }
                }
                KeyCode::Char('k') => {
                    if state.selected >= 1 {
                        state.selected -= 1;
                    }
                }
                KeyCode::Char('l') | KeyCode::Enter => {
                    state.mode = AppMode::Edit {
                        mode: EditMode::Move,
                        edit_bufs: EditBufs::from_day(&state.days[state.selected]),
                        field: EditField::Date,
                        index: state.selected,
                    }
                }
                KeyCode::Char('+') => {}
                _ => (),
            }
        }
    }

    Ok(false)
}

fn handle_events(state: &mut AppState) -> Result<bool, ()> {
    if event::poll(std::time::Duration::from_millis(50)).map_err(|err| {
        eprintln!("could not poll events: {err}");
    })? {
        match &state.mode {
            AppMode::ListOnly => return handle_events_listonly(state),
            AppMode::Edit { .. } => return handle_events_edit(state),
        }
    }

    Ok(false)
}

fn ui(frame: &mut Frame, state: &AppState) {
    let list_size = 0.5;
    let mut list_area = frame.size();
    list_area.height = (list_area.height as f32 * list_size) as u16;
    frame.render_widget(Clear, frame.size());

    let mut list_active = true;
    if let AppMode::Edit { .. } = &state.mode {
        let edit_pos = Rect {
            x: list_area.x,
            y: list_area.y + list_area.height,
            width: list_area.width,
            height: frame.size().height - list_area.height,
        };
        render_edit_window(frame, &edit_pos, state);
        list_active = false;
    }
    render_list(frame, &list_area, state, list_active);
}

fn load_days(file_path: &str) -> Result<Vec<WorkDay>, ()> {
    let days = std::fs::read_to_string(file_path).map_err(|err| {
        eprintln!("Could not read file {file_path}: {err}");
    })?;

    serde_json::from_str(&days).map_err(|err| {
        eprintln!("Error during parsing of data: {err}");
    })
}

fn main() -> Result<(), ()> {
    let mut state = AppState {
        days: load_days("./work_times.json")?,
        selected: 0,
        mode: AppMode::ListOnly,
    };

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).map_err(|err| {
        eprintln!("Could not create terminal: {err}");
    })?;
    enable_raw_mode().map_err(|err| {
        eprintln!("Could not enable raw mode: {err}");
    })?;
    stdout()
        .execute(EnterAlternateScreen)
        .map_err(|err| eprintln!("could not enter alternate screen: {err}"))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|frame| ui(frame, &state)).map_err(|err| {
            eprintln!("Could not draw UI: {err}");
            should_quit = true;
        })?;
        should_quit = handle_events(&mut state)?;
    }

    disable_raw_mode().map_err(|err| {
        eprintln!("Could not disable raw mode: {err}");
    })?;
    stdout()
        .execute(LeaveAlternateScreen)
        .map_err(|err| eprintln!("could not leave alternate screen: {err}"))?;
    Ok(())
}
