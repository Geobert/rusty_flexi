use chrono::{NaiveTime, Duration, NaiveDate};
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
            start: NaiveTime::from_hms(9, 0, 0),
            end: NaiveTime::from_hms(18, 0, 0),
            pause: Duration::minutes(30).num_minutes(),
            status: Default::default()
        }
    }
}

impl PartialEq for FlexDay {
    fn eq(&self, other: &FlexDay) -> bool {
        self.date == other.date
    }
}

impl FlexDay {
    pub fn new(date: NaiveDate, settings: Settings) -> FlexDay {
//        let default = match settings.is_exception(date) {
//            true => ,
//            false => settings.week_sched.default,
//        };
//        FlexDay
//            { date: date, start: default., end: end, pause: pause.num_minutes(), status: status }
        FlexDay {..Default::default()}
    }
}
