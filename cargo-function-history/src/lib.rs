use std::{cell::RefCell, io::stdout, rc::Rc};

use crate::app::ui;
use app::{state::AppState, App, AppReturn};
use crossterm::event::{self, Event, KeyCode};
use eyre::Result;
use keys::Key;
use tui::{backend::CrosstermBackend, Terminal};
use tui_input::backend::crossterm as input_backend;

pub mod app;
pub mod keys;
pub fn start_ui(app: Rc<RefCell<App>>) -> Result<()> {
    // Configure Crossterm backend for tui
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    loop {
        let mut app = app.borrow_mut();

        // Render
        terminal.draw(|rect| ui::draw(rect, &mut app))?;

        // Handle inputs
        if let Event::Key(key) = event::read()? {
            match &mut app.state() {
                AppState::Editing => match key.code {
                    KeyCode::Enter => {
                        app.run_command();
                        app.input_buffer.reset();
                    }
                    KeyCode::Esc => {
                        app.state = AppState::Looking;
                    }
                    KeyCode::Delete => {
                        app.input_buffer.reset();
                    }
                    _ => {
                        input_backend::to_input_request(Event::Key(key))
                            .and_then(|req| app.input_buffer.handle(req));
                    }
                },
                _ => {
                    let result = app.do_action(Key::from(key));
                    // Check if we should exit
                    if result == AppReturn::Exit {
                        break;
                    }
                }
            }
        }
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
