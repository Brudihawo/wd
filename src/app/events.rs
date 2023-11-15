use chrono::{Local, NaiveTime};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::app_common::{AppMode, AppState, Message, StatsState};
use crate::editor::{EditBufs, EditField, EditMode};
use crate::stat::{total_stats, weekly_stats};
use crate::work_day::{DayType, WorkDay, Break};

pub fn handle_events_help(state: &mut AppState) -> Result<bool, ()> {
    if let Event::Key(key) = event::read().map_err(|err| {
        eprintln!("Could not read event: {err}");
    })? {
        if key.kind == event::KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('x') => return Ok(true),
                KeyCode::Char('?') | KeyCode::Esc => state.help_popup = None,
                KeyCode::Char('j') => state.help_popup = Some(state.help_popup.unwrap() + 1),
                KeyCode::Char('k') => {
                    if state.help_popup.unwrap() > 0 {
                        state.help_popup = Some(state.help_popup.unwrap() - 1);
                    }
                }
                KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                    state.help_popup = Some(state.help_popup.unwrap() + 5);
                }
                KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                    if state.help_popup.unwrap() > 5 {
                        state.help_popup = Some(state.help_popup.unwrap() - 5);
                    } else {
                        state.help_popup = Some(0);
                    }
                }
                _ => (),
            }
        }
    }
    Ok(false)
}

pub fn handle_events_stats(state: &mut AppState) -> Result<bool, ()> {
    if let Event::Key(key) = event::read().map_err(|err| {
        eprintln!("Could not read event: {err}");
    })? {
        if key.kind == event::KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('s') | KeyCode::Esc => state.statistics = None,
                KeyCode::Char('j') => {
                    state
                        .statistics
                        .iter_mut()
                        .map(|val| {
                            // TODO: wrap scrolling - somehow
                            val.scroll += 1
                        })
                        .next();
                }
                KeyCode::Char('k') => {
                    state
                        .statistics
                        .iter_mut()
                        .map(|val| {
                            if val.scroll > 0 {
                                val.scroll -= 1;
                            }
                        })
                        .next();
                }
                KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                    state
                        .statistics
                        .iter_mut()
                        .map(|val| {
                            // TODO: wrap scrolling - somehow
                            val.scroll += 5;
                        })
                        .next();
                }
                KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                    state
                        .statistics
                        .iter_mut()
                        .map(|val| {
                            // TODO: wrap scrolling - somehow
                            if val.scroll > 5 {
                                val.scroll -= 5;
                            } else {
                                val.scroll = 0;
                            }
                        })
                        .next();
                }
                _ => (),
            }
        }
    }

    Ok(false)
}

pub fn handle_events(state: &mut AppState) -> Result<bool, ()> {
    if event::poll(std::time::Duration::from_millis(50)).map_err(|err| {
        eprintln!("could not poll events: {err}");
    })? {
        if state.help_popup.is_some() {
            return handle_events_help(state);
        } else if state.statistics.is_some() {
            return handle_events_stats(state);
        } else {
            match &state.mode {
                AppMode::ListOnly => return handle_events_listonly(state),
                AppMode::Edit { .. } => return handle_events_edit(state),
            }
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
                KeyCode::Char('?') => state.help_popup = Some(0),
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('w') => state.write()?,
                KeyCode::Char('x') => {
                    state.write()?;
                    return Ok(true);
                }
                KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                    for _ in 0..5 {
                        state.selected = state.next_day();
                    }
                }
                KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                    for _ in 0..5 {
                        state.selected = state.prev_day();
                    }
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
                KeyCode::Char('s') => {
                    state.statistics = {
                        if let Some(total) = total_stats(&state.days) {
                            Some(StatsState {
                                weekly: weekly_stats(&state.days),
                                total,
                                scroll: 0,
                            })
                        } else {
                            None
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
            let (edit_bufs, field, e_mode, _index) = match &mut state.mode {
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
                    KeyCode::Char('?') => state.help_popup = Some(0),
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Char('w') => state.write()?,
                    KeyCode::Char('s') => match (&*edit_bufs).try_into() {
                        Ok(val) => {
                            state.days[*selected] = val;
                            state.message =
                                Message::Info(String::from("WorkDay parsed successfully"));
                            state.days.sort_by_key(|day| day.date);
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
                            *e_mode = EditMode::Insert;
                        }
                    }
                    _ => (),
                },
                EditMode::Insert => match key.code {
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
