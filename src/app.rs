use crate::editor::{EditBufs, EditDayType, EditField, EditMode};
use crate::work_day::{Break, DayType, WorkDay};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::{Color, Frame, Rect, Style, Stylize},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use chrono::{Local, NaiveTime};

pub const ORANGE: Color = Color::Rgb(255, 140, 0);

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

pub struct AppState {
    pub file_path: String,
    pub days: Vec<WorkDay>,
    pub selected: Option<usize>,
    pub mode: AppMode,
    pub message: Message,
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

pub fn handle_events(state: &mut AppState) -> Result<bool, ()> {
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

fn handle_events_listonly(state: &mut AppState) -> Result<bool, ()> {
    if let Event::Key(key) = event::read().map_err(|err| {
        eprintln!("Could not read event: {err}");
    })? {
        if key.kind == event::KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('w') => state.write()?,
                KeyCode::Char('x') => {
                    state.write()?;
                    return Ok(true);
                }
                KeyCode::Char('d') => {
                    if let Some(selected) = state.selected.as_mut() {
                        let removed = state.days.remove(*selected);
                        if *selected == state.days.len() {
                            *selected = state.days.len() - 1;
                        }
                        state.message = Message::Info(format!("Removed entry of {}", removed.date))
                    }
                }
                KeyCode::Char('j') => state.selected = state.next_day(),
                KeyCode::Char('k') => state.selected = state.prev_day(),
                KeyCode::Char('l') | KeyCode::Enter => {
                    if let Some(selected) = state.selected {
                        state.mode = AppMode::Edit {
                            mode: EditMode::Move,
                            edit_bufs: EditBufs::from(&state.days[selected]),
                            field: EditField::Date,
                            index: selected,
                        }
                    }
                }
                KeyCode::Char('+') => {
                    state.days.push(WorkDay {
                        date: Local::now().naive_local().date(),
                        day_type: DayType::Present {
                            start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                            end: NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
                            brk: Break {
                                start: NaiveTime::from_hms_opt(11, 30, 0).unwrap(),
                                end: NaiveTime::from_hms_opt(12, 00, 0).unwrap(),
                            },
                        },
                    });
                    let selected = state.days.len() - 1;
                    state.selected = Some(selected);
                    state.mode = AppMode::Edit {
                        mode: EditMode::Move,
                        edit_bufs: EditBufs::from(&state.days[selected]),
                        field: EditField::Date,
                        index: selected,
                    }
                }
                _ => (),
            }
        }
    }

    Ok(false)
}

fn handle_events_edit(state: &mut AppState) -> Result<bool, ()> {
    if let Event::Key(key) = event::read().map_err(|err| {
        eprintln!("Could not read event: {err}");
    })? {
        if key.kind == event::KeyEventKind::Press {
            let next = state.next_day();
            let prev = state.prev_day();
            let (edit_bufs, field, e_mode, index) = match &mut state.mode {
                AppMode::ListOnly => unreachable!(),
                AppMode::Edit {
                    edit_bufs,
                    field,
                    mode,
                    index,
                } => (edit_bufs, field, mode, index),
            };
            let selected = if let Some(selected) = state.selected.as_mut() {
                selected
            } else {
                unreachable!()
            };

            match e_mode {
                EditMode::Move => match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Char('w') => state.write()?,
                    KeyCode::Char('s') => match (&*edit_bufs).try_into() {
                        Ok(val) => {
                            state.days[*selected] = val;
                            state.message =
                                Message::Info(String::from("WorkDay parsed successfully"));
                        }
                        Err(err) => state.message = Message::Error(err),
                    },
                    KeyCode::Char('x') => {
                        state.write()?;
                        return Ok(true);
                    }
                    KeyCode::Tab => {
                        *selected = next.unwrap();
                        state.mode = AppMode::Edit {
                            mode: EditMode::Move,
                            edit_bufs: EditBufs::from(&state.days[*selected]),
                            field: EditField::Date,
                            index: *selected,
                        }
                    }
                    KeyCode::BackTab => {
                        *selected = prev.unwrap();
                        state.mode = AppMode::Edit {
                            mode: EditMode::Move,
                            edit_bufs: EditBufs::from(&state.days[*selected]),
                            field: EditField::Date,
                            index: *selected,
                        }
                    }
                    KeyCode::Char('j') => *field = field.next(edit_bufs.day_type),
                    KeyCode::Char('k') => *field = field.prev(edit_bufs.day_type),
                    KeyCode::Esc | KeyCode::Char('h') => state.mode = AppMode::ListOnly,
                    KeyCode::Enter | KeyCode::Char('l') => {
                        if *field == EditField::DayType {
                            edit_bufs.day_type = edit_bufs.day_type.next();
                        } else {
                            *e_mode = EditMode::Edit;
                        }
                    }
                    _ => (),
                },
                EditMode::Edit => match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        *e_mode = EditMode::Move;
                    }
                    KeyCode::Char(c) => {
                        assert_ne!(*field, EditField::DayType);
                        if c.is_ascii() {
                            let (buf, cur) = edit_bufs.entry_mut(*field);
                            if *cur < 64 {
                                buf[*cur as usize] = c as u8;
                                *cur += 1;
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        assert_ne!(*field, EditField::DayType);
                        let (_, cur) = edit_bufs.entry_mut(*field);
                        if *cur > 0 {
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

pub mod render {
    use super::*;

    fn render_edit_window(frame: &mut Frame, pos: &Rect, state: &AppState) {
        if let AppMode::Edit {
            field,
            edit_bufs,
            mode: e_mode,
            index,
        } = &state.mode
        {
            let day = &state.days[*index];

            use EditField::*;
            let field_index = match *field {
                Date => 0,
                DayType => 1,
                Start => 2,
                End => 3,
                BreakStart => 4,
                BreakEnd => 5,
            };

            let mut names = vec![ListItem::new("Date"), ListItem::new("Status")];
            let mut bufs = vec![
                edit_bufs.text(Date),
                match edit_bufs.day_type {
                    EditDayType::Present => "Present",
                    EditDayType::Sick => "Sick",
                    EditDayType::Unofficial { has_break } => {
                        if has_break {
                            "Unofficial (with break)"
                        } else {
                            "Unofficial (no break)"
                        }
                    }
                },
            ];

            match edit_bufs.day_type {
                EditDayType::Present => {
                    names.extend_from_slice(&[
                        ListItem::new("Start"),
                        ListItem::new("End"),
                        ListItem::new("Break Start"),
                        ListItem::new("Break End"),
                    ]);
                    bufs.extend_from_slice(&[
                        edit_bufs.text(Start),
                        edit_bufs.text(End),
                        edit_bufs.text(BreakStart),
                        edit_bufs.text(BreakEnd),
                    ]);
                }
                EditDayType::Sick => (),
                EditDayType::Unofficial { has_break } => {
                    names.extend_from_slice(&[ListItem::new("Start"), ListItem::new("End")]);
                    bufs.extend_from_slice(&[edit_bufs.text(Start), edit_bufs.text(End)]);
                    if has_break {
                        bufs.extend_from_slice(&[
                            edit_bufs.text(BreakStart),
                            edit_bufs.text(BreakEnd),
                        ]);
                        names.extend_from_slice(&[
                            ListItem::new("Break Start"),
                            ListItem::new("Break End"),
                        ]);
                    }
                }
            }

            let bufs = bufs
                .iter()
                .enumerate()
                .map(|(i, buf)| {
                    if i == field_index {
                        ListItem::new([*buf, "_"].concat())
                    } else {
                        ListItem::new(*buf)
                    }
                })
                .collect::<Vec<_>>();

            frame.render_widget(
                Block::default()
                    .title(format!("Edit {}", day.date.to_string()))
                    .borders(Borders::ALL),
                *pos,
            );

            let mut inner = *pos;
            inner.x += 1;
            inner.y += 1;
            inner.width -= 2;
            inner.height -= 2;

            let mut buf_area = inner;
            let mut name_area = inner;
            name_area.width = 17;

            buf_area.x += name_area.width;
            buf_area.width -= name_area.width;

            frame.render_stateful_widget(
                List::new(names).highlight_style(Style::default().bold().fg(match e_mode {
                    EditMode::Move => ORANGE,
                    EditMode::Edit => Color::LightBlue,
                })),
                name_area,
                &mut ListState::default().with_selected(Some(field_index)),
            );
            frame.render_stateful_widget(
                List::new(bufs).highlight_style(Style::default().bold().fg(match e_mode {
                    EditMode::Move => ORANGE,
                    EditMode::Edit => Color::LightBlue,
                })),
                buf_area,
                &mut ListState::default().with_selected(Some(field_index)),
            );
        } else {
            unreachable!()
        }
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
            &mut ListState::default().with_selected(state.selected),
        );
    }

    fn render_message_area(frame: &mut Frame, area: &Rect, message: &Message) {
        match &message {
            Message::Info(msg) => frame.render_widget(
                Block::default()
                    .title(msg.clone())
                    .style(Style::default().fg(Color::LightBlue).bold()),
                *area,
            ),
            Message::Error(msg) => frame.render_widget(
                Block::default()
                    .title(msg.clone())
                    .style(Style::default().fg(Color::LightYellow).bold()),
                *area,
            ),
            Message::None => (),
        }
    }

    pub fn ui(frame: &mut Frame, state: &AppState) {
        let list_size = 0.5;
        let mut list_area = frame.size();
        list_area.height = (list_area.height as f32 * list_size) as u16;
        let edit_area = Rect {
            x: list_area.x,
            y: list_area.y + list_area.height,
            width: list_area.width,
            height: frame.size().height - list_area.height - 1,
        };
        let mut msg_area = frame.size();
        msg_area.y += msg_area.height - 1;
        msg_area.height = 1;

        frame.render_widget(Clear, frame.size());

        let mut list_active = true;
        if let AppMode::Edit { .. } = &state.mode {
            render_edit_window(frame, &edit_area, state);
            list_active = false;
        }

        render_list(frame, &list_area, state, list_active);
        render_message_area(frame, &msg_area, &state.message);
    }
}
