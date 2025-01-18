use std::{fs, path::Path};

pub fn setup(path: String) {
    if Path::new(&path).exists() {
        fs::remove_file(path).expect("Failed to remove test file");
    }
}
