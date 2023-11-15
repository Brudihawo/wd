use crate::editor::{EditBufs, EditDayType, EditField, EditMode};
use crate::work_day::{Break, DayType, WorkDay};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::{Color, Frame, Rect, Style, Stylize},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use chrono::{Local, NaiveTime};

const ORANGE: Color = Color::Rgb(255, 140, 0);

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
    pub days: Vec<WorkDay>,
    pub selected: usize,
    pub mode: AppMode,
    pub message: Message,
}

pub fn handle_events_listonly(state: &mut AppState) -> Result<bool, ()> {
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
                        edit_bufs: EditBufs::from(&state.days[state.selected]),
                        field: EditField::Date,
                        index: state.selected,
                    }
                }
                KeyCode::Char('+') => {
                    state.selected = state.days.len();
                    state.days.push(WorkDay {
                        date: Local::now().naive_local().date(),
                        day_type: DayType::Present {
                            start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                            end: NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
                            brk: Break {
                                break_start: NaiveTime::from_hms_opt(11, 30, 0).unwrap(),
                                break_end: NaiveTime::from_hms_opt(12, 00, 0).unwrap(),
                            },
                        },
                    });
                    state.mode = AppMode::Edit {
                        mode: EditMode::Move,
                        edit_bufs: EditBufs::from(&state.days[state.selected]),
                        field: EditField::Date,
                        index: state.selected,
                    }
                }
                _ => (),
            }
        }
    }

    Ok(false)
}

pub fn handle_events_edit(state: &mut AppState) -> Result<bool, ()> {
    if let Event::Key(key) = event::read().map_err(|err| {
        eprintln!("Could not read event: {err}");
    })? {
        if key.kind == event::KeyEventKind::Press {
            let (edit_bufs, field, e_mode, index) = match &mut state.mode {
                AppMode::ListOnly => unreachable!(),
                AppMode::Edit {
                    edit_bufs,
                    field,
                    mode,
                    index,
                } => (edit_bufs, field, mode, index),
            };

            const ORANGE: Color = Color::Rgb(255, 140, 0);
            match e_mode {
                EditMode::Move => match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Char('j') => *field = field.next(edit_bufs.day_type),
                    KeyCode::Char('k') => *field = field.prev(edit_bufs.day_type),
                    KeyCode::Char('s') => {
                        // TODO: error messages
                        match (&*edit_bufs).try_into() {
                            Ok(val) => {
                                state.days[*index] = val;
                                state.message =
                                    Message::Info(String::from("WorkDay parsed successfully"));
                            }
                            Err(err) => state.message = Message::Error(err),
                        }
                    }
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
            &mut ListState::default().with_selected(Some(state.selected)),
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
