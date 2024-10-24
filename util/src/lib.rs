use std::path::Path;

pub fn extract_last_directory_name(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        if let Some(last_segment) = parent.file_name() {
            return last_segment.to_str().map(|s| s.to_string());
        }
    }
    None
}
