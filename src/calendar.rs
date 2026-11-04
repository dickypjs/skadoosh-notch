use chrono::{Datelike, Local, NaiveDate};

pub struct CalendarGrid {
    pub year: i32,
    pub month: u32,
    pub days_in_month: u32,
    pub first_weekday: u32, // 0 = Mon, 6 = Sun
    pub today_day: u32,
}

impl CalendarGrid {
    pub fn current() -> Self {
        let now = Local::now().date_naive();
        let year = now.year();
        let month = now.month();
        let today_day = now.day();

        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let first_weekday = first_day.weekday().num_days_from_monday();

        let next_month = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        };

        let days_in_month = (next_month - first_day).num_days() as u32;

        Self {
            year,
            month,
            days_in_month,
            first_weekday,
            today_day,
        }
    }
}
