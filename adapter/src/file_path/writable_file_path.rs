use shared::error::{AppError, AppResult};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WritableFilePath(PathBuf);

impl WritableFilePath {
    pub fn new(path: impl Into<PathBuf>) -> AppResult<Self> {
        let path = path.into();

        if !path.is_file() {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("ファイル '{}' は存在しません", path.display()),
            )));
        }

        let metadata = fs::metadata(&path)?;
        if metadata.permissions().readonly() {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("ファイル '{}' に書き込み権限がありません", path.display()),
            )));
        }

        Ok(Self(path))
    }

    pub fn read_content(&self) -> AppResult<String> {
        std::fs::read_to_string(&self.0).map_err(AppError::Io)
    }
}

impl TryFrom<String> for WritableFilePath {
    type Error = AppError;

    fn try_from(path: String) -> Result<Self, Self::Error> {
        Self::new(path)
    }
}

impl std::ops::Deref for WritableFilePath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_temp_file(content: &str) -> NamedTempFile {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(&temp_file, content).unwrap();
        temp_file
    }

    #[test]
    fn writable_file_path_creates_instance_with_writable_file() {
        // ===== Arrange =====
        let temp_file = create_temp_file("test content");
        let path = temp_file.path().to_path_buf();

        // ===== Act =====
        let result = WritableFilePath::new(path.clone());

        // ===== Assert =====
        assert!(result.is_ok());
        let writable_file = result.unwrap();
        assert_eq!(writable_file.to_str().unwrap(), path.to_str().unwrap());
    }

    #[test]
    fn writable_file_path_fails_with_nonexistent_file() {
        // ===== Arrange =====
        let nonexistent_path = PathBuf::from("nonexistent_file.txt");

        // ===== Act =====
        let result = WritableFilePath::new(nonexistent_path);

        // ===== Assert =====
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Io(io_error) => {
                assert_eq!(io_error.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("Expected Io error with NotFound kind"),
        }
    }

    #[test]
    fn writable_file_path_fails_with_readonly_file() {
        // ===== Arrange =====
        let temp_file = create_temp_file("test content");
        let path = temp_file.path().to_path_buf();

        // ファイルを読み取り専用に設定
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&path, perms).unwrap();

        // ===== Act =====
        let result = WritableFilePath::new(path);

        // ===== Assert =====
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Io(io_error) => {
                assert_eq!(io_error.kind(), std::io::ErrorKind::PermissionDenied);
            }
            _ => panic!("Expected Io error with PermissionDenied kind"),
        }
    }

    #[test]
    fn writable_file_path_fails_with_directory_path() {
        // ===== Arrange =====
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        // ===== Act =====
        let result = WritableFilePath::new(dir_path);

        // ===== Assert =====
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Io(io_error) => {
                assert_eq!(io_error.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("Expected Io error with NotFound kind"),
        }
    }

    #[test]
    fn writable_file_path_read_content_returns_file_content() {
        // ===== Arrange =====
        let content = "Hello, World!\nThis is a test file.";
        let temp_file = create_temp_file(content);
        let path = temp_file.path().to_path_buf();
        let writable_file = WritableFilePath::new(path).unwrap();

        // ===== Act =====
        let result = writable_file.read_content();

        // ===== Assert =====
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    #[test]
    fn writable_file_path_read_content_returns_empty_string_for_empty_file() {
        // ===== Arrange =====
        let temp_file = create_temp_file("");
        let path = temp_file.path().to_path_buf();
        let writable_file = WritableFilePath::new(path).unwrap();

        // ===== Act =====
        let result = writable_file.read_content();

        // ===== Assert =====
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
}
