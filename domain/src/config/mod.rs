use crate::config::{destination_directory_path::DestinationDirectoryPath, source_directory_path::SourceDirectoryPath, weekday::WeekDay};

pub(crate) mod destination_directory_path;
pub(crate) mod source_directory_path;
pub(crate) mod weekday;

pub struct Config {
    pub source_directory_path: SourceDirectoryPath,
    pub dest_directory_path: DestinationDirectoryPath,
    pub weekday: WeekDay,
}
