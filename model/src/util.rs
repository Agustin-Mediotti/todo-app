use std::path::PathBuf;

pub fn is_completed(completed: bool) -> String {
    if completed {
        "[x]".to_owned()
    } else {
        "[]".to_owned()
    }
}

pub fn get_data_path(filename: &str) -> Option<PathBuf> {
    let base = dirs::data_dir().expect("Error on location");
    let app_dir = base.join("todo-app");

    std::fs::create_dir_all(&app_dir).expect("No se pudo crear el directorio de datos");

    Some(app_dir.join(filename))
}
