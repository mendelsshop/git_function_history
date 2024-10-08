use std::{cell::RefCell, io::stdout, path::PathBuf, process::exit, rc::Rc, time::Duration};

use app::{state::AppState, ui, App, AppReturn};
use crossterm::event::{self, Event, KeyCode};
use eyre::Result;
use keys::Key;
use ratatui::{backend::CrosstermBackend, style::Modifier, Terminal};

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
    let mut x = false;
    loop {
        let mut app = app.borrow_mut();

        // Render
        terminal.draw(|rect| {
            // check if we have enough space to draw
            if rect.area().width < 15 || rect.area().height < 15 {
                x = true
            } else {
                ui::draw(rect, &mut app)
            }
        })?;

        if x {
            break;
        }
        // Handle inputs
        if let Ok(true) = event::poll(Duration::from_millis(100)) {
            if let Event::Key(key) = event::read()? {
                match &mut app.state() {
                    AppState::Editing => match key.code {
                        KeyCode::Enter => {
                            app.run_command();
                            app.history_index = app.history.len();
                            app.reset_and_save()
                        }
                        KeyCode::Esc => {
                            let style = app
                                .input_buffer
                                .cursor_style()
                                .remove_modifier(Modifier::REVERSED);
                            app.input_buffer.set_cursor_style(style);
                            app.state = AppState::Looking;
                        }
                        KeyCode::Up => {
                            app.reset_and_save();
                            app.scroll_up();
                        }
                        KeyCode::Down => {
                            app.reset_and_save();
                            app.scroll_down();
                        }
                        _ => {
                            app.input_buffer.input(key);
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
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;
    if x {
        eprintln!("Not enough space to draw");
        exit(1)
    }

    Ok(())
}

fn get_history_dir() -> Result<PathBuf> {
    let mut path =
        dirs::data_local_dir().ok_or_else(|| eyre::eyre!("Could not find data local dir"))?;
    path.push("cargo_function_history");
    path.push("history.txt");
    Ok(path)
}
