mod common;
use model::common::Task;
use todo_app::app::App;

#[test]
fn add_correct_task() {
    common::setup("user_data".to_owned());
    let mut app = App::new().expect("error instanciating app");
    app.clean_tasks().expect("error while removing all tasks");
    assert_eq!(
        app.add_task(
            Task::new(
                app.tasks.len(),
                String::from("Hello World"),
                String::from("hello world")
            )
            .expect("error creating new task")
        )
        .unwrap(),
        ()
    );
}

#[test]
fn display_tasks() {
    common::setup("user_data".to_owned());
    let mut app = App::new().expect("error instanciating app");
    app.clean_tasks().expect("error while removing all tasks");
    app.add_task(
        Task::new(
            app.tasks.len(),
            String::from("Hello World"),
            String::from("Hello World"),
        )
        .expect("error creating new task"),
    )
    .expect("error while adding a new task");
    assert_eq!(app.tasks_into_string(), "Hello World [] Hello World \n");
}

#[test]
fn remove_task() {
    common::setup("user_data".to_owned());
    let mut app = App::new().expect("error instanciating app");
    app.clean_tasks().expect("error while removing all tasks");
    app.add_task(
        Task::new(
            app.tasks.len(),
            String::from("Hello World"),
            String::from("Hello World"),
        )
        .expect("error creating new task"),
    )
    .expect("error while adding a new task");
    app.remove_task(0).expect("error while removing task");
    assert_eq!(app.tasks_into_string(), "");
}

#[test]
fn remove_all_tasks() {
    common::setup("user_data".to_owned());
    let mut app = App::new().expect("error instanciating app");
    app.clean_tasks().expect("error while removing all tasks");
    app.add_task(
        Task::new(
            app.tasks.len(),
            String::from("Hello World"),
            String::from("Hello World"),
        )
        .expect("error creating new task"),
    )
    .expect("error while adding a new task");
    app.clean_tasks().expect("error while removing all tasks");
    assert_eq!(app.tasks_into_string(), "");
}
