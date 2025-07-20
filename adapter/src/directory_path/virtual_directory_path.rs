use std::{fs, path::PathBuf};
use shared::error::{AppError, AppResult};

use super::writable_directory_path::WritableDirectoryPath;

#[derive(Debug)]
pub struct VirtualDirectoryPath(PathBuf);

impl VirtualDirectoryPath {
    pub fn new(path: impl Into<PathBuf>) -> AppResult<Self>{
        let path = path.into();

        if path.exists() {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "指定されたパスは既に存在します"
            )));
        }

        Ok(Self(path))
    }

    pub fn create_writable_directory_path(self) -> AppResult<WritableDirectoryPath> {
        fs::create_dir_all(self.0.clone())?;
        WritableDirectoryPath::new(self.0)
    }

    pub fn to_str(&self) -> AppResult<&str> {
        self.0.to_str().ok_or_else(|| AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "パスに無効な文字が含まれています"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn virtual_directory_path_creates_instance_with_nonexistent_path() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent_dir");

        // ===== Act =====
        let result = VirtualDirectoryPath::new(nonexistent_path.clone());

        // ===== Assert =====
        assert!(result.is_ok());
        let virtual_path = result.unwrap();
        assert_eq!(virtual_path.to_str().unwrap(), nonexistent_path.to_str().unwrap());
    }

    #[test]
    fn virtual_directory_path_fails_with_existing_path() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let existing_path = temp_dir.path().join("existing_dir");
        fs::create_dir(&existing_path).unwrap();

        // ===== Act =====
        let result = VirtualDirectoryPath::new(existing_path);

        // ===== Assert =====
        assert!(result.is_err());
    }

    #[test]
    fn virtual_directory_path_creates_writable_directory_path_successfully() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let virtual_path = temp_dir.path().join("new_dir");
        let virtual_dir = VirtualDirectoryPath::new(virtual_path.clone()).unwrap();

        // ===== Act =====
        let result = virtual_dir.create_writable_directory_path();

        // ===== Assert =====
        assert!(result.is_ok());
        let writable_dir = result.unwrap();
        assert!(virtual_path.exists());
        assert!(virtual_path.is_dir());
        assert!(!fs::metadata(&virtual_path).unwrap().permissions().readonly());
        assert_eq!(writable_dir.to_str().unwrap(), virtual_path.to_str().unwrap());
    }
}
