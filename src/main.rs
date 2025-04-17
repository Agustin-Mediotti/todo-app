use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use model::util::get_data_path;
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::fs::create_dir_all;
use std::io;
use todo_app::app::App;

pub mod app;
pub mod banner;
pub mod ui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    enable_raw_mode()?;

    let json_path = get_data_path("data.json").expect("could not get data directory");

    if let Some(parent) = json_path.parent() {
        if let Err(e) = create_dir_all(parent) {
            eprintln!("Error creating directory {:?}: {}", parent, e);
            std::process::exit(1);
        }
    }

    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::with_json(json_path.to_str().unwrap()).unwrap_or_default();
    let result = app.run(&mut terminal);

    if let Err(err) = ratatui::try_restore() {
        eprintln!("failed to restore terminal: {}", err);
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;
    result
}
