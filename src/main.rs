use model::common::Task;
use todo_app::app::App;

pub mod app;
pub mod banner;
pub mod ui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let mut app = App::new().unwrap_or_default();

    app.clean_tasks()?;
    let task = Task::new(app.index(), "Analizar".to_string())?;
    let task2 = Task::new(app.index(), "Leer".to_string())?;
    let task3 = Task::new(app.index(), "Escuchar".to_string())?;
    app.add_task(task)?;
    app.add_task(task2)?;
    app.add_task(task3)?;

    app.change_task_done(1)?;
    let result = app.run(terminal);
    if let Err(err) = ratatui::try_restore() {
        eprintln!("failed to restore terminal: {}", err);
    }
    result
}
