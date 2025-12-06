use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Read a file's contents as a string
pub fn read_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))
}

/// Write content to a file, creating parent directories if needed
pub fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))
}

/// Extract the file stem (name without extension) from a path
pub fn get_file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Schema")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_file_stem() {
        assert_eq!(get_file_stem(&PathBuf::from("user.json")), "user");
        assert_eq!(get_file_stem(&PathBuf::from("/path/to/schema.json")), "schema");
        assert_eq!(get_file_stem(&PathBuf::from("data")), "data");
    }
}
