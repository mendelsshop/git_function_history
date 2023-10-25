use std::collections::BTreeMap;

use function_history_backend_thread::types::Status;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation},
    Frame,
};

use crate::app::App;

use super::CommandResult;

pub fn draw(rect: &mut Frame, app: &mut App) {
    let size = rect.size();
    // check if we have enough space to draw
    if size.width < 10 || size.height < 10 {
        panic!("Not enough space to draw");
    }
    let main_window = draw_main();
    let mut whole_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(size.height - 1)].as_ref())
        .split(size)[1];
    rect.render_widget(main_window, size);
    whole_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(whole_chunks.width - 2),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(whole_chunks)[1];
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(whole_chunks.height - 4),
                Constraint::Length(2),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(whole_chunks);
    app.get_result();
    draw_body(
        app,
        *body_chunks.get(0).expect("could not get area to draw"),
        rect,
    );
    rect.render_widget(
        app.input_buffer.widget(),
        *body_chunks.get(1).expect("could not get area to draw"),
    );
    let status = draw_status(app.status());
    rect.render_widget(
        status,
        *body_chunks.get(2).expect("could not get area to draw"),
    );
}

fn draw_body(app: &mut App, mut pos: Rect, frame: &mut Frame) {
    let top = match &app.cmd_output {
        CommandResult::History(history) => {
            let metadata = history.get_metadata();
            let metadata = BTreeMap::from_iter(metadata.iter());
            let metadata: Vec<Line> = metadata
                .iter()
                .map(|x| Line::from(format!("{}: {}\n", x.0, x.1)))
                .collect();
            Some(
                Paragraph::new(metadata)
                    .style(Style::default().fg(Color::LightCyan))
                    .block(Block::default().style(Style::default().fg(Color::White))),
            )
        }
        _ => None,
    };
    let tick_text: Vec<Line> = match &app.cmd_output {
        CommandResult::None => match app.status {
            Status::Loading => vec![Line::from(format!(
                "Loading{}",
                ".".repeat(
                    ((std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_millis()
                        / 100)
                        % 4) as usize
                )
            ))],
            _ => vec![Line::from(
                "Please enter some commands to search for a function.",
            )],
        },
        a => a
            .to_string()
            .split('\n')
            .map(|s| Line::from(format!("{s}\n")))
            .collect(),
    };
    app.scroll_state = app.scroll_state.content_length(tick_text.len());
    let body = Paragraph::new(tick_text)
        .style(Style::default().fg(Color::LightCyan))
        .scroll(app.scroll_pos)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .style(Style::default().fg(Color::White)),
        );
    if let Some(top) = top {
        let mut top_pos = pos;
        top_pos.height = 4;
        pos.height -= 4;
        pos.y += 4;
        frame.render_widget(top, top_pos);
    }
    app.body_height = pos.height;
    frame.render_widget(body, pos);

    frame.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        pos,
        &mut app.scroll_state,
    )
}

fn draw_main<'a>() -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(" Cargo Function History ")
        .border_style(Style::default().fg(Color::White))
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::White))
}

fn draw_status<'a>(status: &Status) -> Paragraph<'a> {
    Paragraph::new(vec![Line::from(Span::raw(status.to_string()))])
        .style(Style::default().fg(Color::LightCyan))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .borders(Borders::BOTTOM)
                .style(Style::default().fg(Color::White)),
        )
}
