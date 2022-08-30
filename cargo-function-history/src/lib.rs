use std::io::stdout;
use std::rc::Rc;
use std::{cell::RefCell, time::Duration};

use app::{App, AppReturn};
use eyre::Result;
use inputs::InputEvent;
use inputs::{events::Events, key::Key};
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
        let result = match events.next()? {
            InputEvent::Input(key) => app.do_action(key),
            InputEvent::Tick => AppReturn::Continue,
        };
        // Check if we should exit
        if result == AppReturn::Exit {
            break;
        } else if let AppReturn::TextEdit(x, y) = result {
            terminal.set_cursor(x, y)?;
            crossterm::terminal::disable_raw_mode()?;
            terminal.show_cursor()?;
            let mut inputs = String::new();
            std::io::stdin().read_line(&mut inputs)?;
            crossterm::terminal::enable_raw_mode()?;
            terminal.hide_cursor()?;
            app.input_buffer.push_str(&inputs);
            app.do_action(Key::Enter);
            // crossterm::terminal::enable_raw_mode()?;
        }
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
