use std::{fs::{self, File}, io::Read, path::Path};

use sha2::{Digest, Sha256};
use shared::error::{AppError, AppResult};

pub struct FileSystem;

impl FileSystem {
    pub fn copy_all_data_under_the_directory_with_hash_verification(from: &Path, to: &Path) -> AppResult<()> {
        Self::copy_directory_recursively(from, to)
    }

    fn copy_directory_recursively(from: &Path, to: &Path) -> AppResult<()> {
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let entry_path = entry.path();
            let rel_path = entry_path.strip_prefix(from)?;
            let dest_path = to.join(rel_path);
            
            if entry.file_type()?.is_dir() {
                fs::create_dir_all(&dest_path)?;
                Self::copy_directory_recursively(&entry_path, &dest_path)?;
            } else {
                fs::copy(entry.path(), dest_path.as_path())?;

                let entry_hash = Self::calculate_hash_from_file_content(&entry_path)?;
                let dest_hash = Self::calculate_hash_from_file_content(&dest_path)?;
                if entry_hash != dest_hash {
                    return Err(AppError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("ハッシュ値が一致しません。: {} -> {}", entry_hash, dest_hash)
                    )));
                }
            }
        }
        Ok(())
    }

    fn calculate_hash_from_file_content(path: &Path) -> AppResult<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn is_path_readonly(path: &Path) -> AppResult<bool> {
        let metadata = fs::metadata(path)?;
        Ok(metadata.permissions().readonly())
    }

    pub fn is_directory_empty(path: &Path) -> AppResult<bool> {
        let mut entries = fs::read_dir(path)?;
        Ok(entries.next().is_none())
    }

    pub fn verify_directory_contents_match(path_1: &Path, path_2: &Path) -> AppResult<bool> {
        let list_1 = Self::list_relative_paths(path_1)?;
        let list_2 = Self::list_relative_paths(path_2)?;
        Ok(list_1 == list_2)
    }

    fn list_relative_paths(base: &Path) -> AppResult<Vec<String>> {
        let mut list = Vec::new();
        for entry in fs::read_dir(base)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path == base {
                continue;
            }
            let rel = entry_path.strip_prefix(base).unwrap().to_path_buf();
            list.push(rel.to_string_lossy().to_string());
        }
        list.sort();
        Ok(list)
    }

    pub fn clear_directory_contents<P: AsRef<Path>>(dir: P) -> AppResult<()> {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                Self::clear_directory_contents(&path)?;
                fs::remove_dir(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        }
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn copy_all_data_under_the_directory_with_hash_verification_successfully_copies_files_and_directories() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");
        
        fs::create_dir(&source_dir).unwrap();
        fs::create_dir(&dest_dir).unwrap();
        
        // Create test files
        let test_file1 = source_dir.join("file1.txt");
        let test_file2 = source_dir.join("file2.txt");
        let test_subdir = source_dir.join("subdir");
        let test_file3 = test_subdir.join("file3.txt");
        
        fs::create_dir(&test_subdir).unwrap();
        
        File::create(&test_file1).unwrap().write_all(b"content1").unwrap();
        File::create(&test_file2).unwrap().write_all(b"content2").unwrap();
        File::create(&test_file3).unwrap().write_all(b"content3").unwrap();

        // ===== Act =====
        let result = FileSystem::copy_all_data_under_the_directory_with_hash_verification(&source_dir, &dest_dir);

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(dest_dir.join("file1.txt").exists());
        assert!(dest_dir.join("file2.txt").exists());
        assert!(dest_dir.join("subdir").exists());
        assert!(dest_dir.join("subdir").join("file3.txt").exists());
    }

    #[test]
    fn copy_all_data_under_the_directory_with_hash_verification_returns_error_when_source_directory_does_not_exist() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let non_existent_source = temp_dir.path().join("non_existent");
        let dest_dir = temp_dir.path().join("dest");
        
        fs::create_dir(&dest_dir).unwrap();

        // ===== Act =====
        let result = FileSystem::copy_all_data_under_the_directory_with_hash_verification(&non_existent_source, &dest_dir);

        // ===== Assert =====
        assert!(result.is_err());
    }

    #[test]
    fn is_path_readonly_returns_true_for_readonly_file() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("readonly.txt");
        File::create(&test_file).unwrap();
        
        let mut perms = fs::metadata(&test_file).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&test_file, perms).unwrap();

        // ===== Act =====
        let result = FileSystem::is_path_readonly(&test_file);

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn is_path_readonly_returns_false_for_writable_file() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("writable.txt");
        File::create(&test_file).unwrap();

        // ===== Act =====
        let result = FileSystem::is_path_readonly(&test_file);

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn is_directory_empty_returns_true_for_empty_directory() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let empty_dir = temp_dir.path().join("empty");
        fs::create_dir(&empty_dir).unwrap();

        // ===== Act =====
        let result = FileSystem::is_directory_empty(&empty_dir);

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn is_directory_empty_returns_false_for_non_empty_directory() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let non_empty_dir = temp_dir.path().join("non_empty");
        fs::create_dir(&non_empty_dir).unwrap();
        
        File::create(non_empty_dir.join("file.txt")).unwrap();

        // ===== Act =====
        let result = FileSystem::is_directory_empty(&non_empty_dir);

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn verify_directory_contents_match_returns_true_for_identical_directories() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let dir1 = temp_dir.path().join("dir1");
        let dir2 = temp_dir.path().join("dir2");
        
        fs::create_dir(&dir1).unwrap();
        fs::create_dir(&dir2).unwrap();
        
        // Create identical structure
        fs::create_dir(dir1.join("subdir")).unwrap();
        fs::create_dir(dir2.join("subdir")).unwrap();
        
        File::create(dir1.join("file1.txt")).unwrap().write_all(b"content").unwrap();
        File::create(dir2.join("file1.txt")).unwrap().write_all(b"content").unwrap();
        
        File::create(dir1.join("subdir").join("file2.txt")).unwrap().write_all(b"content").unwrap();
        File::create(dir2.join("subdir").join("file2.txt")).unwrap().write_all(b"content").unwrap();

        // ===== Act =====
        let result = FileSystem::verify_directory_contents_match(&dir1, &dir2);

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn verify_directory_contents_match_returns_false_for_different_directories() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let dir1 = temp_dir.path().join("dir1");
        let dir2 = temp_dir.path().join("dir2");
        
        fs::create_dir(&dir1).unwrap();
        fs::create_dir(&dir2).unwrap();
        
        // Create different structure
        File::create(dir1.join("file1.txt")).unwrap();
        File::create(dir2.join("file2.txt")).unwrap();

        // ===== Act =====
        let result = FileSystem::verify_directory_contents_match(&dir1, &dir2);

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn calculate_hash_from_file_content_returns_consistent_hash_for_same_content() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap().write_all(b"test content").unwrap();

        // ===== Act =====
        let hash1 = FileSystem::calculate_hash_from_file_content(&test_file);
        let hash2 = FileSystem::calculate_hash_from_file_content(&test_file);

        // ===== Assert =====
        assert!(hash1.is_ok());
        assert!(hash2.is_ok());
        assert_eq!(hash1.unwrap(), hash2.unwrap());
    }

    #[test]
    fn calculate_hash_from_file_content_returns_different_hash_for_different_content() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        
        File::create(&file1).unwrap().write_all(b"content1").unwrap();
        File::create(&file2).unwrap().write_all(b"content2").unwrap();

        // ===== Act =====
        let hash1 = FileSystem::calculate_hash_from_file_content(&file1);
        let hash2 = FileSystem::calculate_hash_from_file_content(&file2);

        // ===== Assert =====
        assert!(hash1.is_ok());
        assert!(hash2.is_ok());
        assert_ne!(hash1.unwrap(), hash2.unwrap());
    }

    #[test]
    fn clear_directory_contents_removes_all_files_and_subdirectories() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test_dir");
        fs::create_dir(&test_dir).unwrap();
        
        // Create files and subdirectories
        let file1 = test_dir.join("file1.txt");
        let file2 = test_dir.join("file2.txt");
        let subdir = test_dir.join("subdir");
        let subfile = subdir.join("subfile.txt");
        
        fs::create_dir(&subdir).unwrap();
        File::create(&file1).unwrap().write_all(b"content1").unwrap();
        File::create(&file2).unwrap().write_all(b"content2").unwrap();
        File::create(&subfile).unwrap().write_all(b"subcontent").unwrap();

        // ===== Act =====
        let result = FileSystem::clear_directory_contents(&test_dir);

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(FileSystem::is_directory_empty(&test_dir).unwrap());
    }

    #[test]
    fn clear_directory_contents_returns_error_when_directory_does_not_exist() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let non_existent_dir = temp_dir.path().join("non_existent");

        // ===== Act =====
        let result = FileSystem::clear_directory_contents(&non_existent_dir);

        // ===== Assert =====
        assert!(result.is_err());
    }

    #[test]
    fn clear_directory_contents_works_with_empty_directory() {
        // ===== Arrange =====
        let temp_dir = TempDir::new().unwrap();
        let empty_dir = temp_dir.path().join("empty_dir");
        fs::create_dir(&empty_dir).unwrap();

        // ===== Act =====
        let result = FileSystem::clear_directory_contents(&empty_dir);

        // ===== Assert =====
        assert!(result.is_ok());
        assert!(FileSystem::is_directory_empty(&empty_dir).unwrap());
    }
}