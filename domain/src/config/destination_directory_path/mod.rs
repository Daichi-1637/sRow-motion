use adapter::directory_path::{
    virtual_directory_path::VirtualDirectoryPath, writable_directory_path::WritableDirectoryPath,
};
use chrono::Local;
use shared::error::AppResult;

use crate::config::destination_directory_path::path_template_renderer::PathTemplateRenderer;

mod path_template_renderer;

pub struct DestinationDirectoryPath(WritableDirectoryPath);

impl DestinationDirectoryPath {
    pub fn new(path: String) -> AppResult<Self> {
        let template = VirtualDirectoryPath::new(path)?;
        let writable_dir = PathTemplateRenderer::new(template)
            .render(&Local::now())?
            .create_writable_directory_path()?;
        Ok(Self(writable_dir))
    }
}

impl std::ops::Deref for DestinationDirectoryPath {
    type Target = WritableDirectoryPath;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
