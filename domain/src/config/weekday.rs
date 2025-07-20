use shared::error::AppError;
use chrono::{DateTime, Local};
use chrono::Datelike;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeekDay {
    Sunday = 0,
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
}

impl WeekDay {
    pub fn matches_weekday(&self, date: &DateTime<Local>) -> bool {
        date.weekday().num_days_from_sunday() == self.clone() as u32
    }
}

impl TryFrom<String> for WeekDay {
    type Error = AppError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Mon" => Ok(WeekDay::Monday),
            "Tue" => Ok(WeekDay::Tuesday),
            "Wed" => Ok(WeekDay::Wednesday),
            "Thu" => Ok(WeekDay::Thursday),
            "Fri" => Ok(WeekDay::Friday),
            "Sat" => Ok(WeekDay::Saturday),
            "Sun" => Ok(WeekDay::Sunday),
            _ => Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "無効な曜日が指定されています"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, TimeZone};

    #[test]
    fn weekday_matches_thursday() {
        // ===== Arrange =====
        let date = Local.with_ymd_and_hms(2025, 6, 19, 0, 0, 0).unwrap(); // 木曜日

        // ===== Act =====
        let result = WeekDay::Thursday.matches_weekday(&date);

        // ===== Assert =====
        assert!(result);
    }

    #[test]
    fn weekday_creation_from_string() {
        // ===== Arrange =====
        let weekday = "Thu";

        // ===== Act =====
        let result = WeekDay::try_from(weekday.to_string()).unwrap();

        // ===== Assert =====
        assert_eq!(result, WeekDay::Thursday);
    }

    #[test]
    fn weekday_creation_from_invalid_string() {
        // ===== Arrange =====
        let weekday = "Invalid";

        // ===== Act =====
        let result = WeekDay::try_from(weekday.to_string());

        // ===== Assert =====
        assert!(result.is_err());
    }
}