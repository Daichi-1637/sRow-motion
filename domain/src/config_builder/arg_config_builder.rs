use shared::error::AppResult;

use crate::{
    config::{
        destination_directory_path::DestinationDirectoryPath,
        source_directory_path::SourceDirectoryPath,
        weekday::WeekDay,
        Config,
    },
    config_builder::ConfigBuilder,
};

pub struct ArgConfigBuilder {
    source_directory_path: String,
    destination_directory_path: String,
    weekday: String
}

impl ArgConfigBuilder {
    pub fn new(source_directory_path: String, destination_directory_path: String, weekday: String) -> AppResult<Self> {
        Ok(Self {
            source_directory_path,
            destination_directory_path,
            weekday
        })
    }
}

impl ConfigBuilder for ArgConfigBuilder {
    fn build(&self) -> AppResult<Config> {
        Ok(Config {
            source_directory_path: SourceDirectoryPath::new(self.source_directory_path.clone())?,
            dest_directory_path: DestinationDirectoryPath::new(self.destination_directory_path.clone())?,
            weekday: WeekDay::try_from(self.weekday.clone())?,
        })
    }
}