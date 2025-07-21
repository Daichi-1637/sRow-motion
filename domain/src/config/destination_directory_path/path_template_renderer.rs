use adapter::directory_path::virtual_directory_path::VirtualDirectoryPath;
use chrono::{DateTime, Datelike, Local};
use shared::error::{AppError, AppResult};

trait PadLeft {
    fn pad_left(&self, width: usize, pad_char: char) -> String;
}

impl PadLeft for u32 {
    fn pad_left(&self, width: usize, pad_char: char) -> String {
        let num_str = self.to_string();
        if num_str.len() >= width {
            num_str
        } else {
            let padding = pad_char.to_string().repeat(width - num_str.len());
            padding + &num_str
        }
    }
}

#[derive(Debug)]
pub struct PathTemplateRenderer {
    template: VirtualDirectoryPath,
}

impl PathTemplateRenderer {
    pub fn new(template: VirtualDirectoryPath) -> Self {
        Self { template }
    }

    pub fn render(&self, date: &DateTime<Local>) -> AppResult<VirtualDirectoryPath> {
        let rendered_template = self
            .template
            .to_str()?
            .replace("{yyyy}", &date.year().to_string())
            .replace("{mm}", &date.month().pad_left(2, '0'))
            .replace("{dd}", &date.day().pad_left(2, '0'));

        if rendered_template.contains("{") || rendered_template.contains("}") {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "想定されていない文字列が括弧で囲われています: {}",
                    rendered_template
                ),
            )));
        }

        VirtualDirectoryPath::new(rendered_template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn pad_left() {
        assert_eq!(1u32.pad_left(2, '0'), "01");
        assert_eq!(10u32.pad_left(2, '0'), "10");
        assert_eq!(100u32.pad_left(2, '0'), "100");
    }

    #[test]
    fn path_template_rendering_failure_when_invalid_template() {
        // ===== Arrange =====
        let template = VirtualDirectoryPath::new("/test/files/{yyyy}/{invalid}/{dd}").unwrap();
        let renderer = PathTemplateRenderer::new(template);
        let date = Local.with_ymd_and_hms(2024, 3, 14, 0, 0, 0).unwrap();

        // ===== Act =====
        let result = renderer.render(&date);

        // ===== Assert =====
        assert!(result.is_err());
    }

    #[test]
    fn path_template_rendering_success_when_valid_template() {
        // ===== Arrange =====
        let template = VirtualDirectoryPath::new("/test/files/{yyyy}/{mm}/{dd}").unwrap();
        let renderer = PathTemplateRenderer::new(template);
        let date = Local.with_ymd_and_hms(2024, 3, 14, 0, 0, 0).unwrap();

        // ===== Act =====
        let result = renderer.render(&date);

        // ===== Assert =====
        assert!(result.is_ok());
    }
}
