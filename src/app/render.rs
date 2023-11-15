use ratatui::text::{Line, Text};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use static_assertions::const_assert_eq;

use crate::app_common::Message;

use super::*;

pub fn render_application(frame: &mut Frame, state: &AppState) {
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

    if state.statistics.is_some() {
        let stat_inset = 8;
        let mut popup_area = frame.size();
        popup_area.x += stat_inset;
        popup_area.y += stat_inset;
        popup_area.width -= stat_inset * 2;
        popup_area.height -= stat_inset * 2;

        render_statistics_popup(frame, &popup_area, state.statistics.as_ref().unwrap());
    }

    if state.help_popup.is_some() {
        let help_inset = 8;
        let mut popup_area = frame.size();
        popup_area.x += help_inset;
        popup_area.y += help_inset;
        popup_area.width -= help_inset * 2;
        popup_area.height -= help_inset * 2;

        render_help_popup(frame, &popup_area, &state);
    }
}

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
                EditMode::Move => EDIT_MOVE_CLR,
                EditMode::Insert => EDIT_INS_CLR,
            })),
            name_area,
            &mut ListState::default().with_selected(Some(field_index)),
        );
        frame.render_stateful_widget(
            List::new(bufs).highlight_style(Style::default().bold().fg(match e_mode {
                EditMode::Move => EDIT_MOVE_CLR,
                EditMode::Insert => EDIT_INS_CLR,
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
                    Style::default().fg(MOVE_CLR)
                } else {
                    Style::default().fg(Color::Gray)
                }),
        )
        .highlight_symbol(">>")
        .highlight_style(Style::default().fg(MOVE_CLR).bold()),
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

pub fn render_help_popup(frame: &mut Frame, area: &Rect, state: &AppState) {
    frame.render_widget(Clear, *area);
    frame.render_widget(
        Block::default()
            .title("Help")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(HELP_CLR).bold()),
        *area,
    );

    let mut list_mode_area = *area;
    list_mode_area.x += 1;
    list_mode_area.y += 1;
    list_mode_area.width -= 2;
    list_mode_area.height = 9;

    let mut edit_mode_move_area = list_mode_area;
    edit_mode_move_area.y += list_mode_area.height + 1;
    edit_mode_move_area.height = 11;

    let list_text = [
        "            ?  open help",
        "            q  quit",
        "            w  write to disk",
        "            x  write to disk and quit",
        "            d  delete current selection",
        "          j/k  scroll down/up",
        "      <c-u/d>  scroll down/up by 5",
        "    l/<enter>  enter edit mode - move on selection",
        "            +  add new entry",
    ]
    .as_slice();

    let edit_move_text = [
        "            ?  open help",
        "            q  quit",
        "            w  write to disk",
        "            s  save current entry",
        "            x  write and quit",
        "        <tab>  next entry",
        "      <s-tab>  previous entry",
        "          j/k  field below/above",
        "      <esc>/h  go back to list mode",
        "    <enter>/l  edit current field (edit mode - insert)",
    ]
    .as_slice();

    let edit_insert_text = [
        "<enter>/<esc>  finish editing field (edit mode - move)",
        "any character  type in current field",
        "      <bcksp>  delete a character",
    ]
    .as_slice();

    const_assert_eq!(SCROLL_AMT, 5);
    // Update Help Text if SCROLL_AMT has changed
    let help_text = [
        "          q/x  quit",
        "      ?/<esc>  close help popup",
        "          j/k  scroll down/up",
        "      <c-u/d>  scroll down/up by 5",
    ]
    .as_slice();

    let help_segments = [
        (list_text, MOVE_CLR, "Move Mode"),
        (edit_move_text, EDIT_MOVE_CLR, "Edit Mode - Move"),
        (edit_insert_text, EDIT_INS_CLR, "Edit Mode - Insert"),
        (help_text, HELP_CLR, "Help Popup"),
    ];

    let mut lines = Vec::new();
    for (text, color, title) in help_segments {
        lines.push(ListItem::new(title).bold().fg(color));
        lines.extend(text.iter().map(|l| ListItem::new(*l).fg(color)));
        lines.push(ListItem::new("").bold().fg(color));
    }
    lines.extend((0..SCROLL_AMT - 1).map(|_| ListItem::new("")));

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width - 5,
        height: area.height - 2,
    };

    let scrollbar_area = Rect {
        x: inner.x + inner.width,
        y: inner.y,
        width: 1,
        height: inner.height,
    };

    let num_lines = lines.len();
    let pos = state.help_popup.unwrap() % num_lines;

    frame.render_stateful_widget(
        List::new(lines)
            .highlight_style(Style::default().bold().fg(Color::White))
            .highlight_symbol("> "),
        inner,
        &mut ListState::default().with_selected(Some(pos)),
    );

    frame.render_stateful_widget(
        Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight),
        scrollbar_area,
        &mut ScrollbarState::new(num_lines).position(pos),
    );
}

fn render_statistics_popup(frame: &mut Frame, area: &Rect, stats: &StatsState) {
    frame.render_widget(Clear, *area);
    frame.render_widget(
        Block::default()
            .title("Statistics")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(STAT_CLR).bold()),
        *area,
    );

    let header = format!(
        "{:12} {:10} {:7} {:11} {:9}",
        "Week Start", "Week End", "Hours", "Active Days", "Sick Days"
    )
    .bold()
    .fg(ORANGE);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: 54,
        height: area.height - 2,
    };

    let stat_collect = stats
        .weekly
        .iter()
        .map(|(week_start, stat)| {
            let week_end = week_start.week(chrono::Weekday::Mon).last_day();
            ListItem::new(format!(
                "{:12} {:10} {:7} {:11} {:9}",
                week_start.format("%d.%m.%y"),
                week_end.format("%d.%m.%y"),
                hm_from_duration(stat.work),
                stat.active_days,
                stat.sick_days,
            ))
        })
        .collect::<Vec<_>>();

    let num_lines = stat_collect.len() + 5;
    let pos = stats.scroll % num_lines;

    frame.render_stateful_widget(
        List::new(stat_collect)
            .block(Block::default().title(header))
            .highlight_style(Style::default().bold())
            .highlight_symbol("> "),
        inner,
        &mut ListState::default().with_selected(Some(pos)),
    );

    let scrollbar_area = Rect {
        x: inner.x + inner.width,
        y: inner.y,
        width: 1,
        height: inner.height,
    };

    frame.render_stateful_widget(
        Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight),
        scrollbar_area,
        &mut ScrollbarState::new(num_lines).position(pos),
    );

    let right = Rect {
        x: inner.x + inner.width + 2,
        y: inner.y,
        width: area.width - inner.width - 4,
        height: inner.height,
    };

    let total = Text::from(vec![
        Line::from(vec![
            "Work Time: ".fg(STAT_CLR),
            hm_from_duration(stats.total.work).into(),
        ]),
        Line::from(vec![
            "Avg:".fg(STAT_CLR),
            hm_from_duration(chrono::Duration::minutes(
                stats.total.work.num_minutes() / stats.total.active_days as i64,
            ))
            .into(),
            " / work day".into(),
        ]),
        "".into(),
        Line::from(vec![
            "Break Time:".fg(STAT_CLR),
            hm_from_duration(stats.total.brk).into(),
        ]),
        Line::from(vec![
            "Avg:".fg(STAT_CLR),
            format!(
                "{} / work day",
                hm_from_duration(chrono::Duration::minutes(
                    stats.total.brk.num_minutes() / stats.total.active_days as i64
                ))
            )
            .into(),
        ]),
        Line::from(""),
        Line::from("Days".fg(STAT_CLR).bold()),
        Line::from(vec![
            "Worked: ".fg(STAT_CLR),
            format!(
                "{:4} ({:.1}%)",
                stats.total.active_days,
                stats.total.active_days as f64
                    / (stats.total.sick_days + stats.total.active_days) as f64
                    * 100.0
            )
            .into(),
        ]),
        Line::from(vec![
            "Sick:   ".fg(STAT_CLR),
            format!(
                "{:4} ({:.1}%)",
                stats.total.sick_days,
                stats.total.sick_days as f64
                    / (stats.total.sick_days + stats.total.active_days) as f64
                    * 100.0
            )
            .into(),
        ]),
    ]);
    frame.render_widget(
        Paragraph::new(total).style(Style::default()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(STAT_CLR))
                .title("Total")
                .title_style(Style::default().bold().fg(STAT_CLR)),
        ),
        right,
    );
}


