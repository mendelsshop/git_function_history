use std::io::stdout;
use std::rc::Rc;
use std::{cell::RefCell, time::Duration};

use app::{state::AppState, App, AppReturn};

use eyre::Result;
use inputs::events::Events;
use inputs::InputEvent;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use crate::app::ui;

pub mod app;
pub mod inputs;
pub fn start_ui(app: Rc<RefCell<App>>) -> Result<()> {
    // Configure Crossterm backend for tui
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    // User event handler
    let tick_rate = Duration::from_millis(200);
    let events = Events::new(tick_rate);

    loop {
        let mut app = app.borrow_mut();

        // Render
        terminal.draw(|rect| ui::draw(rect, &mut app))?;

        // Handle inputs
        match &mut app.state() {
            AppState::Editing => {
                match events.next()? {
                    InputEvent::Input(key) => app.do_edit_action(key),
                    InputEvent::Tick => {}
                };
            }
            _ => {
                let result = match events.next()? {
                    InputEvent::Input(key) => app.do_action(key),
                    InputEvent::Tick => AppReturn::Continue,
                };
                // Check if we should exit
                if result == AppReturn::Exit {
                    break;
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
