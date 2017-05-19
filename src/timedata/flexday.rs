use chrono::{NaiveTime, Duration, NaiveDate, Weekday, Datelike};
use std::default::Default;
use settings::Settings;
use std::cmp::PartialEq;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum DayStatus {
    Worked,
    Half,
    Holiday,
    Weekend,
}

impl Default for DayStatus {
    fn default() -> DayStatus { DayStatus::Worked }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct FlexDay {
    pub date: Option<NaiveDate>,
    pub weekday: Option<Weekday>,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub pause: i64,
    // TODO switch to Duration when chrono supports Serialize/Deserialize
    pub status: DayStatus
}

impl Default for FlexDay {
    fn default() -> FlexDay {
        FlexDay {
            date: None,
            weekday: None,
            start: NaiveTime::from_hms(9, 0, 0),
            end: NaiveTime::from_hms(18, 0, 0),
            pause: Duration::minutes(30).num_minutes(),
            status: Default::default()
        }
    }
}

impl PartialEq for FlexDay {
    fn eq(&self, other: &FlexDay) -> bool {
        self.weekday == other.weekday
    }
}

impl FlexDay {
    pub fn new(date:NaiveDate, settings: &Settings) -> FlexDay {
        let default = settings.get_default_day_settings_for(&date);
        FlexDay {
            date: Some(date),
            weekday: Some(date.weekday()),
            start: default.start,
            end: default.end,
            pause: default.pause,
            status: default.status
        }
        //        FlexDay {..Default::default()}
    }
}
