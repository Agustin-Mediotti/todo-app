use model::common::Task;

#[test]
fn set_is_done() {
    let mut task = Task::new(0, String::from("Hello World"), String::from("hello world"))
        .expect("error creating new task");
    task.set_completed();
    assert_eq!(task.completed(), true);
}

#[test]
fn change_valid_text() {
    let mut task = Task::new(0, String::from("Hello World"), String::from("hello world"))
        .expect("error creating new task");
    task.set_description(String::from("New Text"));
    assert_eq!(task.description(), String::from("New Text"));
}
