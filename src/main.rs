use std::error::Error;

use model::common::Task;
use todo_app::client::App;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new()?;
    app.clean_tasks()?;

    let first_task = Task::new(app.index(), String::from("Rust The Book"))?;
    let second_task = Task::new(app.index(), String::from("Rust Cookbook"))?;
    let third_task = Task::new(app.index(), String::from("Rustonomicon"))?;
    app.add_task(first_task)?;
    app.add_task(second_task)?;
    app.add_task(third_task)?;

    print!("{}", app.show_tasks());

    app.change_task_text(0, "Rust is awesome!".to_string())?;
    app.change_task_done(0)?;

    print!("{}", app.show_tasks());
    println!();

    Ok(())
}
