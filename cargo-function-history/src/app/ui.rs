use tui::{backend::Backend, text::{Spans, Span}};
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::Frame;

use crate::app::App;

use super::state::AppState;

pub fn draw<B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let size = rect.size();
    let main_window = draw_main();
    let mut whole_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(1), Constraint::Length(size.height-1),].as_ref()).split(size)[1];
    // println!("{:?}", size);
    rect.render_widget(main_window, size);
    whole_chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Length(1), Constraint::Length(whole_chunks.width-2), Constraint::Length(1),].as_ref()).split(whole_chunks)[1];
    let body_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(whole_chunks.height-4), Constraint::Length(2), Constraint::Length(2)].as_ref())
    .split(whole_chunks);
    
    
    let body = draw_body(app.is_loading(), app.state());
    rect.render_widget(body, body_chunks[0]);
    let input = draw_input( app.state());
    rect.render_widget(input, body_chunks[1]);
    let status = draw_status( app.state());
    rect.render_widget(status, body_chunks[2]);

}


fn draw_body<'a>(loading: bool, state: &AppState) -> Paragraph<'a> {
    let loading_text = if loading { "Loading..." } else { "" };
    let tick_text = if let Some(ticks) = state.count_tick() {
        format!("Tick count: {}", ticks)
    } else {
        String::default()
    };
    Paragraph::new(vec![
        // Spans::from(Span::raw(loading_text)),
        Spans::from(Span::raw(tick_text.clone())),
    ])
    .style(Style::default().fg(Color::LightCyan))
    .block(
        Block::default()
        .borders(Borders::TOP).borders(Borders::BOTTOM)
            .style(Style::default().fg(Color::White))
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

fn draw_input<'a>(state: &AppState) -> Paragraph<'a> {
    let initialized_text = if state.is_initialized() {
        "Initialized"
    } else {
        "Not Initialized !"
    };
    Paragraph::new(vec![
        Spans::from(Span::raw(initialized_text)),
    ])
    .style(Style::default().fg(Color::LightCyan))
    .block(
        Block::default()
        .borders(Borders::TOP).borders(Borders::BOTTOM)
            .style(Style::default().fg(Color::White))
    )
}

fn draw_status<'a>(state: &AppState) -> Paragraph<'a> {
    let initialized_text = if state.is_initialized() {
        "Initialized"
    } else {
        "Not Initialized !"
    };
    Paragraph::new(vec![
        Spans::from(Span::raw(initialized_text)),
    ])
    .style(Style::default().fg(Color::LightCyan))
    .block(
        Block::default()
            .borders(Borders::TOP).borders(Borders::BOTTOM)
            .style(Style::default().fg(Color::White))
    )
}