use crate::{
    config::{
        destination_directory_path::DestinationDirectoryPath,
        source_directory_path::SourceDirectoryPath, weekday::WeekDay, Config,
    },
    config_builder::ConfigBuilder,
};
use adapter::file_path::writable_file_path::WritableFilePath;
use serde::Deserialize;
use shared::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
struct JsonConfig {
    source_directory_path: String,
    destination_directory_path: String,
    weekday: String,
}

pub struct JsonConfigBuilder {
    config_path: WritableFilePath,
}

impl JsonConfigBuilder {
    pub fn new(config_path: &str) -> AppResult<Self> {
        let config_path = WritableFilePath::try_from(config_path.to_string())?;
        Ok(Self { config_path })
    }
}

impl ConfigBuilder for JsonConfigBuilder {
    fn build(&self) -> AppResult<Config> {
        let config_str = self.config_path.read_content()?;
        let config_json: JsonConfig = serde_json::from_str(&config_str)
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        Ok(Config {
            source_directory_path: SourceDirectoryPath::new(config_json.source_directory_path)?,
            dest_directory_path: DestinationDirectoryPath::new(
                config_json.destination_directory_path,
            )?,
            weekday: WeekDay::try_from(config_json.weekday)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    fn create_temp_config_file(content: &str) -> NamedTempFile {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, content).unwrap();
        temp_file
    }

    #[allow(clippy::permissions_set_readonly_false)]
    fn create_temp_directories() -> (TempDir, TempDir) {
        let source_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();
        // source_dir: 読み取り専用
        let mut src_perms = std::fs::metadata(source_dir.path()).unwrap().permissions();
        src_perms.set_readonly(true);
        std::fs::set_permissions(source_dir.path(), src_perms).unwrap();
        // dest_dir: 書き込み可能
        let mut dst_perms = std::fs::metadata(dest_dir.path()).unwrap().permissions();
        dst_perms.set_readonly(false);
        std::fs::set_permissions(dest_dir.path(), dst_perms).unwrap();
        (source_dir, dest_dir)
    }

    #[test]
    fn json_config_builder_creates_instance_with_valid_path() {
        // ===== Arrange =====
        let temp_file = create_temp_config_file("{}");
        let config_path = temp_file.path().to_str().unwrap();

        // ===== Act =====
        let result = JsonConfigBuilder::new(config_path);

        // ===== Assert =====
        assert!(result.is_ok());
        let builder = result.unwrap();
        assert_eq!(builder.config_path.to_str().unwrap(), config_path);
    }

    #[test]
    fn json_config_builder_fails_with_invalid_path() {
        // ===== Arrange =====
        let invalid_path = "/path/does/not/exist";

        // ===== Act =====
        let result = JsonConfigBuilder::new(invalid_path);

        // ===== Assert =====
        assert!(result.is_err());
    }

    #[test]
    fn json_config_builder_builds_config_from_valid_json() {
        // ===== Arrange =====
        let (source_dir, dest_dir) = create_temp_directories();
        // TODO: multi platform
        let source_path = source_dir.path().to_str().unwrap().replace("\\", "/");
        let dest_path = dest_dir
            .path()
            .join("hoge")
            .to_str()
            .unwrap()
            .replace("\\", "/");

        let json_content = format!(
            r#"{{
                "source_directory_path": "{}",
                "destination_directory_path": "{}",
                "weekday": "Thu"
            }}"#,
            source_path, dest_path
        );

        let temp_file = create_temp_config_file(&json_content);
        let config_path = temp_file.path().to_str().unwrap();

        let builder = JsonConfigBuilder::new(config_path).unwrap();

        // ===== Act =====
        let result = builder.build();

        // ===== Assert =====
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.source_directory_path.to_str().unwrap(), source_path);
        assert_eq!(config.dest_directory_path.to_str().unwrap(), dest_path);
        assert_eq!(config.weekday, WeekDay::Thursday);
    }

    #[test]
    fn json_config_builder_fails_with_missing_required_fields() {
        // ===== Arrange =====
        let (source_dir, _) = create_temp_directories();
        let source_path = source_dir.path().to_str().unwrap().replace("\\", "/");

        let json_content = format!(
            r#"{{
                "source_directory_path": "{}"
            }}"#,
            source_path
        );
        let temp_file = create_temp_config_file(&json_content);
        let config_path = temp_file.path().to_str().unwrap();
        let builder = JsonConfigBuilder::new(config_path).unwrap();

        // ===== Act =====
        let result = builder.build();

        // ===== Assert =====
        assert!(result.is_err());
    }
}
