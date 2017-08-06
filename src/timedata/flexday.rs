use chrono::{NaiveTime, Duration, NaiveDate, Weekday, Datelike, Timelike};
use std::default::Default;
use settings::Settings;
use std::fmt::{Display, Result, Formatter};

pub static mut HOLIDAY_DURATION: i64 = 0;

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum DayStatus {
    Worked,
    Holiday,
    Half,
    Weekend,
    Sick
}

impl Default for DayStatus {
    fn default() -> DayStatus { DayStatus::Worked }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct FlexDay {
    pub date: Option<NaiveDate>,
    weekday: Option<Weekday>,
    pub start: NaiveTime,
    pub end: NaiveTime,
    // TODO switch to Duration when chrono supports Serialize/Deserialize
    pub pause: i64,
    pub status: DayStatus
}

impl Default for FlexDay {
    fn default() -> FlexDay {
        FlexDay {
            date: None,
            weekday: None,
            start: NaiveTime::from_hms(9, 0, 0),
            end: NaiveTime::from_hms(17, 0, 0),
            pause: Duration::minutes(30).num_minutes(),
            status: Default::default(),
        }
    }
}

impl Display for FlexDay {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self.status {
            DayStatus::Worked | DayStatus::Half => {
                let pause = Duration::minutes(self.pause);
                let total = Duration::minutes(self.total_minutes());
                write!(f, "{}   {} {:02}/{:02}   {:02}:{:02} -> {:02}:{:02} - {:02}:{:02} = {:02}:{:02}",
                       if self.status == DayStatus::Worked {
                           "N"
                       } else {
                           "h"
                       },
                       match self.weekday {
                           Some(wd) => super::weekday_to_string(wd),
                           None => "???".to_string()
                       },
                       match self.date {
                           Some(date) => date.day(),
                           None => 0u32
                       },
                       match self.date {
                           Some(date) => date.month(),
                           None => 0u32
                       },
                       self.start.hour(),
                       self.start.minute(),
                       self.end.hour(),
                       self.end.minute(),
                       pause.num_hours(),
                       pause.num_minutes() - (pause.num_hours() * 60),
                       total.num_hours(),
                       total.num_minutes() - (total.num_hours() * 60))
            },
            DayStatus::Weekend => {
                write!(f, "{}   {} {:02}/{:02}   --:-- -> --:-- - --:-- = --:--",
                       "W",
                       match self.weekday {
                           Some(wd) => super::weekday_to_string(wd),
                           None => "???".to_string()
                       },
                       match self.date {
                           Some(date) => date.day(),
                           None => 0u32
                       },
                       match self.date {
                           Some(date) => date.month(),
                           None => 0u32
                       })
            },
            DayStatus::Holiday | DayStatus::Sick => {
                let total = Duration::minutes(self.total_minutes());
                write!(f, "{}   {} {:02}/{:02}   --:-- -> --:-- - --:-- = {:02}:{:02}",
                       if self.status == DayStatus::Holiday {
                           "H"
                       } else {
                           "S"
                       },
                       match self.weekday {
                           Some(wd) => super::weekday_to_string(wd),
                           None => "???".to_string()
                       },
                       match self.date {
                           Some(date) => date.day(),
                           None => 0u32
                       },
                       match self.date {
                           Some(date) => date.month(),
                           None => 0u32
                       },
                       total.num_hours(),
                       total.num_minutes() - (total.num_hours() * 60))
            },
        }
    }
}

impl FlexDay {
    pub fn new(date: NaiveDate, settings: &Settings) -> FlexDay {
        let default = settings.get_default_day_settings_for(&date);
        FlexDay {
            date: Some(date),
            weekday: Some(date.weekday()),
            start: default.start,
            end: default.end,
            pause: default.pause,
            status: FlexDay::day_status_for(date.weekday()),
        }
    }

    pub fn status_str(&self) -> String {
        match self.status {
            DayStatus::Worked => "N",
            DayStatus::Holiday => "H",
            DayStatus::Half => "h",
            DayStatus::Sick => "S",
            DayStatus::Weekend => "W",
        }.to_string()
    }

    pub fn total_minutes(&self) -> i64 {
        match self.status {
            DayStatus::Worked | DayStatus::Half =>
                self.end.signed_duration_since(self.start).num_minutes() - self.pause,
            DayStatus::Weekend => 0,
            DayStatus::Holiday | DayStatus::Sick => unsafe { HOLIDAY_DURATION },
        }
    }

    fn day_status_for(wd: Weekday) -> DayStatus {
        match wd {
            Weekday::Sat | Weekday::Sun => DayStatus::Weekend,
            _ => DayStatus::Worked,
        }
    }

    pub fn set_weekday(&mut self, wd: Weekday) {
        self.weekday = Some(wd);
        self.status = FlexDay::day_status_for(wd);
    }

    pub fn weekday(&self) -> Option<Weekday> {
        self.weekday
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_minutes_test() {
        let d: FlexDay = Default::default();
        assert_eq!(d.total_minutes(), 8 * 60 - 30);
    }
}