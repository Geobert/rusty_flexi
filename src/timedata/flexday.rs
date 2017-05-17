use chrono::{ NaiveTime, Weekday, Duration };
use std::default::Default;
use settings::Settings;

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum DayStatus {
    Worked,
    Half,
    Holiday,
    Weekend,
}

impl Default for DayStatus {
    fn default() -> DayStatus { DayStatus::Worked }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct FlexDay {
    pub day: Option<Weekday>,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub pause: i64, // TODO switch to Duration when chrono supports Serialize/Deserialize
    pub status: DayStatus
}

impl Default for FlexDay {
    fn default() -> FlexDay {
        FlexDay { day: None, start: NaiveTime::from_hms(9, 0 , 0), end: NaiveTime::from_hms(18, 0, 0), pause: Duration::minutes(30).num_minutes(), status: Default::default() }
    }
}

impl FlexDay {
    pub fn new(settings: Settings) -> FlexDay {
        // TODO read default from settings
        //FlexDay { day: day, start: start, end: end, pause: pause.num_minutes(), status: status }
        Default::default()
    }
}

