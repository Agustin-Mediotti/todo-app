use model::common::Task;
use std::error::Error;
use todo_app::app::App;

pub mod app;

fn main() -> color_eyre::Result<(), Box<dyn Error>> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let mut app = App::new()?;

    app.clean_tasks().expect("error cleaning tasks");
    let task = Task::new(app.index(), "Analizar".to_string())?;
    let task2 = Task::new(app.index(), "Leer".to_string())?;
    let task3 = Task::new(app.index(), "Escuchar".to_string())?;
    app.add_task(task).expect("error adding task");
    app.add_task(task2).expect("error adding task");
    app.add_task(task3).expect("error adding task");

    app.change_task_done(1)?;
    let result = app.run(terminal)?;
    ratatui::restore();
    Ok(result)
}
