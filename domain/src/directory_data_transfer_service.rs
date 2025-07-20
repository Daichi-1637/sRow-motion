use chrono::{DateTime, Local};
use shared::error::{AppError, AppResult};

use crate::config::Config;

pub struct DirectoryDataTransferService {
    config: Config,
    now: DateTime<Local>,
}

impl DirectoryDataTransferService {
    pub fn new(config: Config) -> Self {
        let now = Local::now();
        Self { config, now }
    }

    #[cfg(test)]
    pub fn with_custom_now(self, now: DateTime<Local>) -> Self {
        Self { config: self.config, now }
    }

    pub fn validate(self) -> AppResult<Self> {
        if !self.config.weekday.matches_weekday(&self.now) {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "今日は指定された曜日ではありません。終了します。: {:?}",
                    self.config.weekday
                ),
            )));
        }

        if !self.config.dest_directory_path.is_empty()? {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "移動先ディレクトリにデータが既に存在するため、処理を終了します",
            )));
        }

        Ok(self)
    }

    pub fn transfer(&self) -> AppResult<()> {
        let result: AppResult<()> = {
            self.config
                .dest_directory_path
                .copy_all_data_from(&self.config.source_directory_path)?;

            match self
                .config
                .dest_directory_path
                .verify_directory_contents_match(&self.config.source_directory_path)?
            {
                true => Ok(()),
                false => Err(AppError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "整合性エラー：コピー内容が一致しません。移動先を削除します。",
                )))
            }
        };

        if let Err(e) = result {
            self.config.dest_directory_path.remove_all()?;
            return Err(e);
        }

        self.config.source_directory_path.remove_all()?;
        println!("ファイルを正常に移動しました。");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_builder::{json_config_builder::JsonConfigBuilder, ConfigBuilder};
    use chrono::TimeZone;
    use std::{fs, path::Path};
    use tempfile::TempDir;

    fn create_test_config_with_weekday(weekday: &str) -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");

        fs::create_dir(&source_dir).unwrap();
        fs::create_dir(&dest_dir).unwrap();

        // ソースディレクトリにファイルを作成
        let source_file = source_dir.join("test.txt");
        fs::write(&source_file, "test content").unwrap();

        let dest_dir = dest_dir.join("hoge");

        // ソースディレクトリを読み取り専用に設定
        let mut source_perms = fs::metadata(&source_dir).unwrap().permissions();
        source_perms.set_readonly(true);
        fs::set_permissions(&source_dir, source_perms).unwrap();

        let json_content = format!(
            r#"{{
                "source_directory_path": "{}",
                "destination_directory_path": "{}",
                "weekday": "{}"
            }}"#,
            source_dir.to_str().unwrap().replace("\\", "/"),
            dest_dir.to_str().unwrap().replace("\\", "/"),
            weekday
        );

        let temp_file = temp_dir.path().join("json_content.json");
        fs::write(&temp_file, json_content).unwrap();

        let builder = JsonConfigBuilder::new(temp_file.to_str().unwrap()).unwrap();
        (builder.build().unwrap(), temp_dir)
    }

    #[test]
    fn directory_data_transfer_service_creates_instance_with_config() {
        // ===== Arrange =====
        let (config, _temp_dir) = create_test_config_with_weekday("Mon");

        // ===== Act =====
        let service = DirectoryDataTransferService::new(config);

        // ===== Assert =====
        assert!(service.config.source_directory_path.exists());
        assert!(service.config.dest_directory_path.exists());
    }

    #[test]
    fn directory_data_transfer_service_validate_fails_on_wrong_weekday() {
        // ===== Arrange =====
        let (config, _temp_dir) = create_test_config_with_weekday("Thu");
        // 2024年1月1日は月曜日
        let now = Local.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let service = DirectoryDataTransferService::new(config).with_custom_now(now);

        // ===== Act =====
        let result = service.validate();

        // ===== Assert =====
        assert!(result.is_err());
    }

    #[test]
    fn directory_data_transfer_service_validate_fails_when_destination_not_empty() {
        // ===== Arrange =====
        let (config, _temp_dir) = create_test_config_with_weekday("Mon");
        // 2024年1月1日は月曜日
        let now = Local.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let service = DirectoryDataTransferService::new(config).with_custom_now(now);

        // 移動先ディレクトリにファイルを作成
        let test_file = service.config.dest_directory_path.join("test.txt");
        let test_file = test_file.to_str().unwrap().replace("\\", "/");
        let test_file = Path::new(&test_file);
        fs::write(&test_file, "test content").unwrap();

        // ===== Act =====
        let result = service.validate();

        // ===== Assert =====
        assert!(result.is_err());
    }

    #[test]
    fn directory_data_transfer_service_transfer_successfully_moves_files() {
        // ===== Arrange =====
        let (config, _temp_dir) = create_test_config_with_weekday("Mon");
        // 2024年1月1日は月曜日
        let now = Local.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let service = DirectoryDataTransferService::new(config).with_custom_now(now);

        // ===== Act =====
        let result = service.transfer();

        // ===== Assert =====
        assert!(result.is_ok());
        // ソースディレクトリが削除されていることを確認
        assert!(service.config.source_directory_path.is_empty().unwrap());
        // 移動先ディレクトリにファイルが存在することを確認
        let dest_file = service.config.dest_directory_path.join("test.txt");
        assert!(!service.config.dest_directory_path.is_empty().unwrap());
        let content = fs::read_to_string(&dest_file).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn directory_data_transfer_service_transfer_removes_destination_on_integrity_error() {
        // ===== Arrange =====
        let (config, _temp_dir) = create_test_config_with_weekday("Mon");
        // 2024年1月1日は月曜日
        let now = Local.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let service = DirectoryDataTransferService::new(config).with_custom_now(now);

        // 移動先ディレクトリに異なるファイルを作成（整合性エラーを引き起こす）
        let dest_file = service.config.dest_directory_path.join("different.txt").to_str().unwrap().replace("\\", "/");
        let dest_file = Path::new(&dest_file);
        println!("移動先ディレクトリ: {:?}", dest_file.to_str());
        fs::write(&dest_file, "different content").unwrap();

        // ===== Act =====
        let result = service.transfer();

        // ===== Assert =====
        assert!(result.is_err());
        assert!(service.config.dest_directory_path.is_empty().unwrap());
        assert!(!service.config.source_directory_path.is_empty().unwrap());
    }
}
