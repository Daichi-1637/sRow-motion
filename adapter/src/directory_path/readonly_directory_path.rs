use infra::file_system::FileSystem;
use shared::error::{AppError, AppResult};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReadonlyDirectoryPath(PathBuf);

impl ReadonlyDirectoryPath {
    pub fn new(path: impl Into<PathBuf>) -> AppResult<Self> {
        let path = path.into();

        if !path.is_dir() {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("ディレクトリ '{}' は存在しません", path.display()),
            )));
        }

        if !FileSystem::is_path_readonly(&path)? {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!(
                    "ディレクトリ '{}' に読み取り専用の権限がありません",
                    path.display()
                ),
            )));
        }

        Ok(Self(path))
    }

    pub fn remove_all(&self) -> AppResult<()> {
        FileSystem::clear_directory_contents(&self.0)
    }

    pub fn is_empty(&self) -> AppResult<bool> {
        FileSystem::is_directory_empty(&self.0)
    }
}

impl TryFrom<String> for ReadonlyDirectoryPath {
    type Error = AppError;

    fn try_from(path: String) -> Result<Self, Self::Error> {
        Self::new(path)
    }
}

impl AsRef<Path> for ReadonlyDirectoryPath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl AsRef<PathBuf> for ReadonlyDirectoryPath {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl std::ops::Deref for ReadonlyDirectoryPath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ReadonlyDirectoryPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_readonly_dir_from_readonly_directory() {
        // ===== Arrange =====
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Set read-only permission for the directory
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&path, perms).unwrap();

        // ===== Act =====
        let source_dir = ReadonlyDirectoryPath::new(path.clone()).unwrap();

        // ===== Assert =====
        assert_eq!(source_dir.to_str().unwrap(), path.to_str().unwrap());
    }

    #[test]
    fn fails_creating_readonly_dir_from_nonexistent_path() {
        // ===== Arrange =====
        let invalid_path = PathBuf::from("/path/does/not/exist");

        // ===== Act =====
        let result = ReadonlyDirectoryPath::new(invalid_path);

        // ===== Assert =====
        assert!(result.is_err());
    }

    #[test]
    fn fails_creating_readonly_dir_from_writable_directory() {
        // ===== Arrange =====
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        // ===== Act =====
        let result = ReadonlyDirectoryPath::new(path.clone());

        // ===== Assert =====
        assert!(result.is_err());
    }

    #[test]
    fn readonly_directory_path_remove_all_successfully_removes_directory() {
        // ===== Arrange =====
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().join("test_dir");
        std::fs::create_dir(&test_dir).unwrap();

        // テストファイルを作成
        let test_file = test_dir.join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        // ディレクトリを読み取り専用に設定
        let mut perms = std::fs::metadata(&test_dir).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&test_dir, perms).unwrap();

        let readonly_dir = ReadonlyDirectoryPath::new(test_dir.to_path_buf()).unwrap();

        // ===== Act =====
        let result = readonly_dir.remove_all();

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(FileSystem::is_directory_empty(&test_dir).unwrap());
    }
}
