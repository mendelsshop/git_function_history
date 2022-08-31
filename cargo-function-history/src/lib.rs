use std::rc::Rc;
use std::{cell::RefCell, time::Duration};
use std::{
    fs::{File, OpenOptions},
    io::{stdout, Write},
    time::Instant,
};

use app::{state::AppState, App, AppReturn};
use crossterm::{
    event::{self, Event},
    terminal,
};
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
        }
        // let mut f = OpenOptions::new()
        // .read(true)
        // .append(true)
        // // .create(true)
        // .open("log").unwrap();
        match &mut app.state() {
            AppState::Editing => {
                // f.write(format!("{:?}", app.input_lines).as_bytes()).unwrap();
                terminal.set_cursor(app.input_lines.0, app.input_lines.1)?;
                terminal.show_cursor()?;
                match read_key(Duration::from_millis(2000)) {
                    Some(key) => {
                        match key {
                            Key::Enter => {
                                app.run_command();
                                app.input_buffer.clear();
                            }
                            Key::Backspace => {
                                // f.write("backspace".as_bytes())?;
                                if app.input_buffer.len() > 0 {
                                    // app.input_lines.0 -= 1;
                                    app.input_buffer.pop();
                                }
                            }
                            Key::Char(c) => {
                                // f.write(format!("{:?}", c).as_bytes())?;
                                // app.input_lines.0 += 1;
                                app.input_buffer.push(c);
                            }
                            Key::Esc => {
                                app.state = AppState::Looking;
                            }
                            _ => {}
                        }
                    }
                    None => {}
                }
            }
            _ => {}
        }
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn read_key(timeout: Duration) -> Option<Key> {
    struct RawModeGuard;
    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            terminal::disable_raw_mode().unwrap();
        }
    }

    terminal::enable_raw_mode().unwrap();
    let _guard = RawModeGuard;
    let start = Instant::now();
    let mut offset = Duration::ZERO;
    while offset <= timeout && event::poll(timeout - offset).unwrap() {
        if let Event::Key(event) = event::read().unwrap() {
            return Some(Key::from(event));
        }
        offset = start.elapsed();
    }
    None
}
