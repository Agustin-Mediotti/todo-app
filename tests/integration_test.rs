mod common;

use common::setup_test_app;
use model::common::Task;

#[test]
fn add_correct_task() {
    let mut test = setup_test_app();
    assert_eq!(
        test.app
            .add_task(
                Task::new(
                    test.app.tasks.len(),
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
    let mut test = setup_test_app();
    test.app
        .add_task(
            Task::new(
                test.app.tasks.len(),
                String::from("Hello World"),
                String::from("Hello World"),
            )
            .expect("error creating new task"),
        )
        .expect("error while adding a new task");
    assert_eq!(
        test.app.tasks_into_string(),
        "Hello World [] Hello World \n"
    );
}

#[test]
fn remove_task() {
    let mut test = setup_test_app();
    test.app
        .add_task(
            Task::new(
                test.app.tasks.len(),
                String::from("Hello World"),
                String::from("Hello World"),
            )
            .expect("error creating new task"),
        )
        .expect("error while adding a new task");
    test.app.remove_task(0).expect("error while removing task");
    assert_eq!(test.app.tasks_into_string(), "");
}

#[test]
fn remove_all_tasks() {
    let mut test = setup_test_app();
    test.app
        .add_task(
            Task::new(
                test.app.tasks.len(),
                String::from("Hello World"),
                String::from("Hello World"),
            )
            .expect("error creating new task"),
        )
        .expect("error while adding a new task");
    test.app
        .clean_tasks()
        .expect("error while removing all tasks");
    assert_eq!(test.app.tasks_into_string(), "");
}
