use std::path::PathBuf;

use clap::Parser;
use domain::{
    config_builder::{arg_config_builder::ArgConfigBuilder, json_config_builder::JsonConfigBuilder, ConfigBuilder},
    directory_data_transfer_service::DirectoryDataTransferService,
};
use shared::error::{AppError, AppResult};

#[derive(Parser)]
#[command(name = "sRow motion")]
#[command(bin_name = "srow")]
#[command(version="0.1")]
#[command(about="Move all date under the specific directory to other directory", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "JSON_FILE")]
    file: Option<PathBuf>,

    #[arg(short, long, value_name = "SOURCE_DIRECTORY", requires = "destination_directory", conflicts_with = "file")]
    source_directory: Option<PathBuf>,

    #[arg(short, long, value_name = "DESTINATION_DIRECTORY", requires = "weekday", conflicts_with = "file")]
    destination_directory: Option<PathBuf>,

    #[arg(short, long, value_name = "WEEKDAY", requires = "source_directory", conflicts_with = "file")]
    weekday: Option<String>,
}

fn main() -> AppResult<()> {
    let cli = Cli::parse();
    
    let config = if let Some(file) = cli.file.as_deref() {
        // 設定ファイルから設定を読み込み
        JsonConfigBuilder::new(file.to_str().unwrap())?.build()?
    } else if let (Some(source), Some(destination), Some(weekday)) = 
        (cli.source_directory, cli.destination_directory, cli.weekday) {
        // コマンドライン引数から設定を構築
        let source_path = source.to_str().unwrap().to_string();
        let destination_path = destination.to_str().unwrap().to_string();
        ArgConfigBuilder::new(source_path, destination_path, weekday)?.build()?
    } else {
        return Err(AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "設定ファイルまたはコマンドライン引数（source_directory, destination_directory, weekday）が必要です"
        )));
    };

    DirectoryDataTransferService::new(config).validate()?.transfer()
}