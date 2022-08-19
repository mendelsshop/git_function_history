use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyEventState, KeyModifiers,
    },
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use git_function_history::{get_function, things::FunctionHistory};
use lazy_static::lazy_static;
use std::{
    env,
    io::{self, Stdout},
    process::exit,
    time::{Duration, Instant},
};
use tui::{
    backend::CrosstermBackend,
    layout::Alignment,
    widgets::{Block, Borders},
    Terminal,
};
lazy_static! {
    static ref BLOCK: tui::widgets::Block<'static> = Block::default()
        .title(" cargo function history ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);
}
fn main() -> Result<(), io::Error> {
    // let block = Block::default().title("cargo function history").borders(Borders::ALL);
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| {
        let size = f.size();
        f.render_widget(BLOCK.clone(), size);
    })?;
    let config = parse_args(&mut terminal);
    let function = match (&config.function_name, &config.file_name) {
        (t, t1) if !t.is_empty() && !t1.is_empty() => {
            get_function(&config.function_name, &config.file_name)
        }
        _ => Ok(FunctionHistory::new(String::new(), Vec::new())),
    };

    if function.is_err() {
        eprintln!("Error:\n\t{}", function.unwrap_err());
        exit(1);
    }
    let function = function.unwrap();
    let _commit_vec = function.history;
    loop {
        terminal.draw(|f| {
            let size = f.size();
            f.render_widget(BLOCK.clone(), size);
        })?;
        if read_key(Duration::from_secs(2))
            == Some(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::empty(),
            })
        {
            break;
        }
    }
    clear_term(&mut terminal);
    Ok(())
}

fn read_key(timeout: Duration) -> Option<KeyEvent> {
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
            return Some(event);
        }
        offset = start.elapsed();
    }
    None
}

fn usage(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> ! {
    clear_term(terminal);
    println!("Usage: cargo function-history [function-name]:[filename] <options>");
    println!("Available options:");
    println!("  --help - show this message");
    exit(1);
}

#[derive(Debug)]
struct Config {
    file_name: String,
    function_name: String,
}

fn parse_args(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Config {
    let mut config = Config {
        file_name: String::new(),
        function_name: String::new(),
    };
    env::args().enumerate().skip(1).for_each(|arg| {
        if arg.0 == 1 {
            match arg.1.split_once(':') {
                Some(string_tuple) => {
                    config.file_name = string_tuple.1.replace('\\', "/");
                    config.function_name = string_tuple.0.to_string();
                }
                _ => {
                    clear_term(terminal);
                    println!("Error:\n\tExpected funtion-name:file-name.\n\tFound function-name.\n\tTip: make sure to separate the function-name and file-name with a colon (:).");
                    exit(1);
                }
            }
        } else {
            match arg.1.as_str() {
                "--help" => {
                    usage(terminal);
                }
                _ => {
                    clear_term(terminal);
                    println!("Error:\n\tUnknown argument: {}\n\tTip: use --help to see available arguments.", arg.1);
                    exit(1);
                }
            }
        }
    });
    config
}

fn clear_term(terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
    disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .expect("Failed to restore terminal");
    terminal.show_cursor().expect("Could not show cursor");
}
