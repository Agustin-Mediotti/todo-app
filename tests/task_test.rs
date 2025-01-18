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
    task.change_text(String::from("New Text"))
        .expect("text is empty");
    assert_eq!(task.description(), String::from("New Text"));
}
