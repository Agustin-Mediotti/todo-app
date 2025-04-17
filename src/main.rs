use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use model::util::get_data_path;
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::{env, io};
use todo_app::app::App;

pub mod app;
pub mod banner;
pub mod ui;

const DEFAULT_DATA_FILENAME: &str = "data.json";

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    enable_raw_mode()?;

    let filename = env::var("JSON_FILE").unwrap_or_else(|_| DEFAULT_DATA_FILENAME.to_string());
    let json_path = get_data_path(&filename).expect("could not get data directory");

    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::with_json(json_path).unwrap_or_default();
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
