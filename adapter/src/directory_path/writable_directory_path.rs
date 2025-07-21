use infra::file_system::FileSystem;
use shared::error::{AppError, AppResult};
use std::path::{Path, PathBuf};

use super::readonly_directory_path::ReadonlyDirectoryPath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WritableDirectoryPath(PathBuf);

impl WritableDirectoryPath {
    pub fn new(path: impl Into<PathBuf>) -> AppResult<Self> {
        let path = path.into();

        if !path.is_dir() {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("ディレクトリ '{}' は存在しません", path.display()),
            )));
        }

        if FileSystem::is_path_readonly(&path)? {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!(
                    "ディレクトリ '{}' に書き込み権限がありません",
                    path.display()
                ),
            )));
        }

        Ok(Self(path))
    }

    pub fn join(&self, path: impl Into<PathBuf>) -> Self {
        let path = self.0.join(path.into());
        Self(path)
    }

    pub fn is_empty(&self) -> AppResult<bool> {
        FileSystem::is_directory_empty(&self.0)
    }

    pub fn copy_all_data_from(&self, source: &ReadonlyDirectoryPath) -> AppResult<()> {
        FileSystem::copy_all_data_under_the_directory_with_hash_verification(
            source.as_path(),
            &self.0,
        )
    }

    pub fn verify_directory_contents_match(&self, other: &Path) -> AppResult<bool> {
        FileSystem::verify_directory_contents_match(&self.0, other)
    }

    pub fn remove_all(&self) -> AppResult<()> {
        FileSystem::clear_directory_contents(&self.0)
    }
}

impl TryFrom<String> for WritableDirectoryPath {
    type Error = AppError;

    fn try_from(path: String) -> Result<Self, Self::Error> {
        Self::new(path)
    }
}

impl TryFrom<PathBuf> for WritableDirectoryPath {
    type Error = AppError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::new(path)
    }
}

impl AsRef<Path> for WritableDirectoryPath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<PathBuf> for WritableDirectoryPath {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl std::ops::Deref for WritableDirectoryPath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for WritableDirectoryPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_writable_dir_from_writable_directory() {
        // ===== Arrange =====
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();

        // ===== Act =====
        let writable_dir = WritableDirectoryPath::new(path.clone()).unwrap();

        // ===== Assert =====
        assert_eq!(writable_dir.to_str().unwrap(), path.to_str().unwrap());
    }

    #[test]
    fn fails_creating_writable_dir_from_nonexistent_path() {
        // ===== Arrange =====
        let path = Path::new("nonexistent");

        // ===== Act =====
        let result = WritableDirectoryPath::new(path);

        // ===== Assert =====
        assert!(result.is_err());
    }

    #[test]
    fn creates_writable_dir_from_readonly_directory() {
        // ===== Arrange =====
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();
        let metadata = std::fs::metadata(&path).unwrap();
        let mut perms = metadata.permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&path, perms).unwrap();

        // ===== Act =====
        let result = WritableDirectoryPath::new(path.clone());

        // ===== Assert =====
        assert!(result.is_err());
    }

    #[test]
    fn writable_directory_path_copy_all_data_from_successfully_copies_files() {
        // ===== Arrange =====
        let temp_dir = tempfile::tempdir().unwrap();
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");

        std::fs::create_dir(&source_dir).unwrap();
        std::fs::create_dir(&dest_dir).unwrap();

        // ソースディレクトリにファイルを作成
        let test_file = source_dir.join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        // ソースディレクトリを読み取り専用に設定
        let mut perms = std::fs::metadata(&source_dir).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&source_dir, perms).unwrap();

        let readonly_source =
            ReadonlyDirectoryPath::new(source_dir.to_string_lossy().to_string()).unwrap();
        let writable_dest = WritableDirectoryPath::new(dest_dir.to_path_buf()).unwrap();

        // ===== Act =====
        let result = writable_dest.copy_all_data_from(&readonly_source);

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(dest_dir.join("test.txt").exists());
        let copied_content = std::fs::read_to_string(dest_dir.join("test.txt")).unwrap();
        assert_eq!(copied_content, "test content");
    }

    #[test]
    fn writable_directory_path_verify_directory_contents_match_returns_true_for_identical_directories(
    ) {
        // ===== Arrange =====
        let temp_dir = tempfile::tempdir().unwrap();
        let dir1 = temp_dir.path().join("dir1");
        let dir2 = temp_dir.path().join("dir2");

        std::fs::create_dir(&dir1).unwrap();
        std::fs::create_dir(&dir2).unwrap();

        // 両方のディレクトリに同じファイルを作成
        let test_file1 = dir1.join("test.txt");
        let test_file2 = dir2.join("test.txt");
        std::fs::write(&test_file1, "test content").unwrap();
        std::fs::write(&test_file2, "test content").unwrap();

        let writable_dir = WritableDirectoryPath::new(dir1.to_path_buf()).unwrap();

        // ===== Act =====
        let result = writable_dir.verify_directory_contents_match(&dir2);

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn writable_directory_path_verify_directory_contents_match_returns_false_for_different_directories(
    ) {
        // ===== Arrange =====
        let temp_dir = tempfile::tempdir().unwrap();
        let dir1 = temp_dir.path().join("dir1");
        let dir2 = temp_dir.path().join("dir2");

        std::fs::create_dir(&dir1).unwrap();
        std::fs::create_dir(&dir2).unwrap();

        // 異なるファイルを作成
        let test_file1 = dir1.join("file1.txt");
        let test_file2 = dir2.join("file2.txt");
        std::fs::write(&test_file1, "content1").unwrap();
        std::fs::write(&test_file2, "content2").unwrap();

        let writable_dir = WritableDirectoryPath::new(dir1.to_path_buf()).unwrap();

        // ===== Act =====
        let result = writable_dir.verify_directory_contents_match(&dir2);

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn writable_directory_path_remove_all_successfully_removes_directory() {
        // ===== Arrange =====
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path().join("test_dir");
        std::fs::create_dir(&test_dir).unwrap();

        // テストファイルを作成
        let test_file = test_dir.join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();

        let writable_dir = WritableDirectoryPath::new(test_dir.to_path_buf()).unwrap();

        // ===== Act =====
        let result = writable_dir.remove_all();

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(FileSystem::is_directory_empty(&test_dir).unwrap());
    }
}
