use std::{cell::RefCell, env, error::Error, process::exit, rc::Rc, sync::mpsc};

use cargo_function_history::{app::App, start_ui};
use function_history_backend_thread::types::{FullCommand, SearchType, Status};
use git_function_history::{FileFilterType, Filter};
use log::info;

fn main() -> Result<(), Box<dyn Error>> {
    simple_file_logger::init_logger!(
        "cargo_function_history",
        simple_file_logger::LogLevel::Trace
    )?;
    info!("Starting cargo function history");
    let (tx_t, rx_m) = mpsc::channel();
    let (tx_m, rx_t) = mpsc::channel();
    function_history_backend_thread::command_thread(rx_t, tx_t, true);
    info!("started command thread");
    let config = parse_args();
    let status = match config.function_name {
        string if string.is_empty() => Status::Ok(None),
        string => {
            tx_m.send(FullCommand::Search(SearchType::new(
                string,
                config.file_type,
                config.filter,
                config.language,
            )))?;
            Status::Loading
        }
    };
    let app = Rc::new(RefCell::new(App::new((tx_m, rx_m), status)));
    start_ui(app)?;
    Ok(())
}

fn usage() -> ! {
    println!("Usage: cargo function-history <function-name<:filename>> <options>");
    println!("Available options:");
    println!("  --help - show this message");
    println!("  --file-absolute - search the exact file with the filename specified after the function name");
    println!("  --file-relative - search any file ending with the filename specified after the function name");
    println!("  --filter-date=<date> - filter to the given date");
    println!("  --filter-commit-hash=<hash> - filter to the given commit hash");
    println!("  --filter-date-range=<date1>:<date2> - filter to the given date range");
    println!("  --lang=[lang] - filter to the given language");
    println!("      Available languages: rust, python, ruby, go, umpl, all");
    println!("      Default: all");
    exit(1);
}

#[derive(Debug)]
struct Config {
    function_name: String,
    filter: Filter,
    file_type: FileFilterType,
    language: git_function_history::languages::Language,
}

fn parse_args() -> Config {
    let mut config = Config {
        function_name: String::new(),
        filter: Filter::None,
        file_type: FileFilterType::None,
        language: git_function_history::languages::Language::All,
    };
    env::args().enumerate().skip(1).for_each(|arg| {
        if arg.0 == 1 {
            if arg.1 == "--help" {
                usage();
            }
            match arg.1.split_once(':') {
                Some(string_tuple) => {
                    config.file_type = FileFilterType::Relative(string_tuple.1.replace('\\', "/"));
                    config.function_name = string_tuple.0.to_string();
                }
                None => {
                    config.function_name = arg.1.to_string();
                }
            }
        } else {
            match arg.1.as_str() {
                "--help" => {
                    usage();
                }
                "--file-absolute" => {
                    match &config.file_type {
                        FileFilterType::None => {
                            eprintln!("Error no file name specified");
                            exit(1);
                        }
                        FileFilterType::Relative(path) => {
                            config.file_type = FileFilterType::Absolute(path.to_string());

                        }
                        _ => {}
                    }
                }
                "--file-relative" => {
                    match &config.file_type {
                        FileFilterType::None => {
                            eprintln!("Error no file name specified");
                            exit(1);
                        }
                        FileFilterType::Absolute(path) => {
                            config.file_type = FileFilterType::Relative(path.to_string());
                        }
                        _ => {}
                    }
                }
                string if string.starts_with("--filter-date=") => {
                    let date = match string.split('=').nth(1) {
                        Some(string) => string,
                        None => {
                            eprintln!("Error no date specified");
                            exit(1);
                        }
                    };
                    config.filter = Filter::Date(date.to_string());
                }
                string if string.starts_with("--filter-commit-hash=") => {
                    let hash = match string.split('=').nth(1) {
                        Some(string) => string,
                        None => {
                            eprintln!("Error no commit hash specified");
                            exit(1);
                        }
                    };
                    config.filter = Filter::CommitHash(hash.to_string());
                }
                string if string.starts_with("--date-range=") => {
                    let date_range = match string.split('=').nth(1) {
                        Some(string) => string,
                        None => {
                            eprintln!("Error no date range specified");
                            exit(1);
                        }
                    };
                    let date_range = match date_range.split_once(':') {
                        Some(string_tuple) => string_tuple,
                        None => {
                            eprintln!("Error no end date specified");
                            exit(1);
                        }
                    };
                    config.filter = Filter::DateRange(date_range.0.to_string(), date_range.1.to_string());
                }
                string if string.starts_with("--lang=") => {
                    let lang = match string.split('=').nth(1) {
                        Some(string) => string,
                        None => {
                            eprintln!("Error no language specified");
                            exit(1);
                        }
                    };
                    match lang {
                        "rust" => {
                            config.language = git_function_history::languages::Language::Rust;
                        }
                        "python" => {
                            config.language = git_function_history::languages::Language::Python;
                        }
                        #[cfg(feature = "c_lang")]
                        "c" => {
                            config.language = git_function_history::languages::Language::C;
                        }
                        "all" => {
                            config.language = git_function_history::languages::Language::All;
                        }
                        _ => {
                            eprintln!("Error invalid language specified");
                            exit(1);
                        }
                    }
                }
                _ => {
                    println!("Error:\n\tUnknown argument: {}\n\tTip: use --help to see available arguments.", arg.1);
                    exit(1);
                }
            }
        }
    });
    config
}

// use std::{
//     error::Error,
//     io,
//     time::{Duration, Instant},
// };

// use crossterm::{
//     event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
//     execute,
//     terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
// };
// use ratatui::{prelude::*, widgets::*};

// #[derive(Default)]
// struct App {
//     pub vertical_scroll_state: ScrollbarState,
//     pub horizontal_scroll_state: ScrollbarState,
//     pub vertical_scroll: usize,
//     pub horizontal_scroll: usize,
// }

// fn main() -> Result<(), Box<dyn Error>> {
//     // setup terminal
//     enable_raw_mode()?;
//     let mut stdout = io::stdout();
//     execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
//     let backend = CrosstermBackend::new(stdout);
//     let mut terminal = Terminal::new(backend)?;

//     // create app and run it
//     let tick_rate = Duration::from_millis(250);
//     let app = App::default();
//     let res = run_app(&mut terminal, app, tick_rate);

//     // restore terminal
//     disable_raw_mode()?;
//     execute!(
//         terminal.backend_mut(),
//         LeaveAlternateScreen,
//         DisableMouseCapture
//     )?;
//     terminal.show_cursor()?;

//     if let Err(err) = res {
//         println!("{err:?}");
//     }

//     Ok(())
// }

// fn run_app<B: Backend>(
//     terminal: &mut Terminal<B>,
//     mut app: App,
//     tick_rate: Duration,
// ) -> io::Result<()> {
//     let mut last_tick = Instant::now();
//     loop {
//         terminal.draw(|f| ui(f, &mut app))?;

//         let timeout = tick_rate
//             .checked_sub(last_tick.elapsed())
//             .unwrap_or_else(|| Duration::from_secs(0));
//         if crossterm::event::poll(timeout)? {
//             if let Event::Key(key) = event::read()? {
//                 match key.code {
//                     KeyCode::Char('q') => return Ok(()),
//                     KeyCode::Char('j') => {
//                         app.vertical_scroll = app.vertical_scroll.saturating_add(1);
//                         app.vertical_scroll_state = app
//                             .vertical_scroll_state
//                             .position(app.vertical_scroll as u16);
//                     }
//                     KeyCode::Char('k') => {
//                         app.vertical_scroll = app.vertical_scroll.saturating_sub(1);
//                         app.vertical_scroll_state = app
//                             .vertical_scroll_state
//                             .position(app.vertical_scroll as u16);
//                     }
//                     KeyCode::Char('h') => {
//                         app.horizontal_scroll = app.horizontal_scroll.saturating_sub(1);
//                         app.horizontal_scroll_state = app
//                             .horizontal_scroll_state
//                             .position(app.horizontal_scroll as u16);
//                     }
//                     KeyCode::Char('l') => {
//                         app.horizontal_scroll = app.horizontal_scroll.saturating_add(1);
//                         app.horizontal_scroll_state = app
//                             .horizontal_scroll_state
//                             .position(app.horizontal_scroll as u16);
//                     }
//                     _ => {}
//                 }
//             }
//         }
//         if last_tick.elapsed() >= tick_rate {
//             last_tick = Instant::now();
//         }
//     }
// }

// fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
//     let size = f.size();

//     // Words made "loooong" to demonstrate line breaking.
//     let s = "Veeeeeeeeeeeeeeeery    loooooooooooooooooong   striiiiiiiiiiiiiiiiiiiiiiiiiing.   ";
//     let mut long_line = s.repeat(usize::from(size.width) / s.len() + 4);
//     long_line.push('\n');

//     let block = Block::default().black();
//     f.render_widget(block, size);

//     let chunks = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints(
//             [
//                 Constraint::Min(1),
//                 Constraint::Percentage(25),
//                 Constraint::Percentage(25),
//                 Constraint::Percentage(25),
//                 Constraint::Percentage(25),
//             ]
//             .as_ref(),
//         )
//         .split(size);

//     let text = vec![
//         Line::from("This is a line "),
//         Line::from("This is a line   ".red()),
//         Line::from("This is a line".on_dark_gray()),
//         Line::from("This is a longer line".crossed_out()),
//         Line::from(long_line.reset()),
//         Line::from("This is a line".reset()),
//         Line::from(vec![
//             Span::raw("Masked text: "),
//             Span::styled(
//                 Masked::new("password", '*'),
//                 Style::default().fg(Color::Red),
//             ),
//         ]),
//         Line::from("This is a line "),
//         Line::from("This is a line   ".red()),
//         Line::from("This is a line".on_dark_gray()),
//         Line::from("This is a longer line".crossed_out()),
//         Line::from(long_line.reset()),
//         Line::from("This is a line".reset()),
//         Line::from(vec![
//             Span::raw("Masked text: "),
//             Span::styled(
//                 Masked::new("password", '*'),
//                 Style::default().fg(Color::Red),
//             ),
//         ]),
//     ];
//     app.vertical_scroll_state = app.vertical_scroll_state.content_length(text.len() as u16);
//     app.horizontal_scroll_state = app
//         .horizontal_scroll_state
//         .content_length(long_line.len() as u16);

//     let create_block = |title| {
//         Block::default()
//             .borders(Borders::ALL)
//             .gray()
//             .title(Span::styled(
//                 title,
//                 Style::default().add_modifier(Modifier::BOLD),
//             ))
//     };

//     let title = Block::default()
//         .title("Use h j k l to scroll ◄ ▲ ▼ ►")
//         .title_alignment(Alignment::Center);
//     f.render_widget(title, chunks[0]);

//     let paragraph = Paragraph::new(text.clone())
//         .gray()
//         .block(create_block("Vertical scrollbar with arrows"))
//         .scroll((app.vertical_scroll as u16, 0));
//     f.render_widget(paragraph, chunks[1]);
//     f.render_stateful_widget(
//         Scrollbar::default()
//             .orientation(ScrollbarOrientation::VerticalRight)
//             .begin_symbol(Some("↑"))
//             .end_symbol(Some("↓")),
//         chunks[1],
//         &mut app.vertical_scroll_state,
//     );
// }
