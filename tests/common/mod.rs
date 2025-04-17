use std::env;

use tempfile::TempDir;
use todo_app::app::App;

pub(crate) const DEFAULT_DATA_FILENAME: &str = "user_data";

#[allow(dead_code)]
pub struct TestApp {
    pub temp_dir: TempDir,
    pub app: App,
}

pub fn setup_test_app() -> TestApp {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let path = temp_dir
        .path()
        .join(env::var("DATA_FILE").unwrap_or_else(|_| DEFAULT_DATA_FILENAME.to_string()));

    let mut app = App::new(path).expect("failed to instantiate app");
    app.clean_tasks().expect("error while removing all tasks");

    TestApp { temp_dir, app }
}
