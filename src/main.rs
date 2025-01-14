use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use model::common::Task;
use ratatui::{prelude::CrosstermBackend, Terminal};
use todo_app::app::App;

pub mod app;
pub mod banner;
pub mod ui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    enable_raw_mode()?;

    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::with_json("data.json").unwrap_or_default();

    app.clean_tasks()?;
    let task = Task::new(
        app.index(),
        "Rustonomicon".to_string(),
        "Per mollis donec dolor imperdiet at turpis lectus.".to_string(),
    )?;
    let task2 = Task::new(
        app.index(),
        "Rust The Book".to_string(),
        "Lorem ipsum odor amet, consectetuer adipiscing elit. Blandit augue luctus hendrerit diam porta nulla semper cursus. ".to_string() 
        + "Placerat ultricies tortor scelerisque eget efficitur neque justo placerat.",
    )?;
    let task3 = Task::new(
        app.index(),
        "Rust Cookbook".to_string(),
        "Lorem ipsum odor amet, consectetuer adipiscing elit.".to_string(),
    )?;
    app.add_task(task)?;
    app.add_task(task2)?;
    app.add_task(task3)?;
    app.change_task_done(1)?;

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
