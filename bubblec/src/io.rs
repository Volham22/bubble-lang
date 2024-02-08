use std::{fs, path::Path};

pub fn load_source_file(path: &Path) -> std::io::Result<String> {
    fs::read_to_string(path)
}
