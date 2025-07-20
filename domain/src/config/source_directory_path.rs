use adapter::directory_path::readonly_directory_path::ReadonlyDirectoryPath;
use shared::error::AppResult;

pub struct SourceDirectoryPath(ReadonlyDirectoryPath);

impl SourceDirectoryPath {
    pub fn new(path: String) -> AppResult<Self> {
        let path = ReadonlyDirectoryPath::try_from(path)?;
        Ok(Self(path))
    }
}

impl std::ops::Deref for SourceDirectoryPath{
    type Target = ReadonlyDirectoryPath;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}