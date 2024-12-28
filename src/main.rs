use color_eyre::{eyre::Ok, Result};
use crossterm::event::{self, Event};
use model::common::Task;
use ratatui::DefaultTerminal;
use todo_app::client::App;

fn main() -> Result<()> {
    color_eyre::install()?;

    // TODO: Better error handling.

    let mut app: App = App::new().expect("error isntanciating App");
    app.clean_tasks().expect("error cleaning tasks");
    let task = Task::new(app.index(), "Rust The Book".to_string())?;
    app.add_task(task).expect("error adding task");

    let terminal = ratatui::init();
    let result = run(terminal, app);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, app: App) -> Result<()> {
    loop {
        terminal.draw(|frame| {
            frame.render_widget(app.show_tasks(), frame.area());
        })?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}
