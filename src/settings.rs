use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Timelike, Weekday};
use savable::Savable;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter, Result};
use std::fs::File;
use std::io::prelude::*;
use timedata::{weekday_to_string, HOLIDAY_DURATION};

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct SettingsDay {
    pub weekday: Weekday,
    pub start: NaiveTime,
    pub end: NaiveTime,
    // TODO switch to Duration when chrono supports Serialize/Deserialize
    pub pause: i64,
}

impl Default for SettingsDay {
    fn default() -> SettingsDay {
        SettingsDay {
            weekday: Weekday::Mon,
            start: NaiveTime::from_hms(9, 0, 0),
            end: NaiveTime::from_hms(17, 0, 0),
            pause: Duration::minutes(30).num_minutes(),
        }
    }
}

impl Display for SettingsDay {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let pause = Duration::minutes(self.pause);
        write!(
            f,
            "{}  {:02}:{:02} -> {:02}:{:02} - {:02}:{:02}",
            weekday_to_string(self.weekday),
            self.start.hour(),
            self.start.minute(),
            self.end.hour(),
            self.end.minute(),
            pause.num_hours(),
            pause.num_minutes() - (pause.num_hours() * 60)
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WeekSchedule {
    pub sched: Vec<SettingsDay>,
}

impl Default for WeekSchedule {
    fn default() -> WeekSchedule {
        WeekSchedule {
            sched: vec![
                SettingsDay {
                    weekday: Weekday::Mon,
                    start: NaiveTime::from_hms(9, 10, 0),
                    end: NaiveTime::from_hms(17, 10, 0),
                    pause: Duration::minutes(30).num_minutes(),
                },
                SettingsDay {
                    weekday: Weekday::Tue,
                    start: NaiveTime::from_hms(9, 10, 0),
                    end: NaiveTime::from_hms(17, 10, 0),
                    pause: Duration::minutes(30).num_minutes(),
                },
                SettingsDay {
                    weekday: Weekday::Wed,
                    start: NaiveTime::from_hms(9, 10, 0),
                    end: NaiveTime::from_hms(17, 10, 0),
                    pause: Duration::minutes(30).num_minutes(),
                },
                SettingsDay {
                    weekday: Weekday::Thu,
                    start: NaiveTime::from_hms(9, 10, 0),
                    end: NaiveTime::from_hms(17, 10, 0),
                    pause: Duration::minutes(30).num_minutes(),
                },
                SettingsDay {
                    weekday: Weekday::Fri,
                    start: NaiveTime::from_hms(9, 10, 0),
                    end: NaiveTime::from_hms(16, 50, 0),
                    pause: Duration::minutes(30).num_minutes(),
                },
            ],
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Offset {
    pub entry: i64, // TODO switch to Duration when chrono supports Serialize
    pub exit: i64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Settings {
    #[serde(default)]
    pub week_sched: WeekSchedule,
    #[serde(default = "default_holidays_per_year")]
    pub holidays_per_year: f32,
    #[serde(default = "default_week_goal")]
    pub week_goal: i64,
    #[serde(default = "default_holiday_duration")]
    pub holiday_duration: i64,
    #[serde(default)]
    pub offsets: Offset,
}

fn default_week_goal() -> i64 {
    Duration::hours(37).num_minutes()
}

fn default_holiday_duration() -> i64 {
    default_week_goal() / 5
}

fn default_holidays_per_year() -> f32 {
    26.0
}

impl Default for Settings {
    fn default() -> Settings {
        let settings = Settings {
            week_sched: WeekSchedule::default(),
            holidays_per_year: default_holidays_per_year(),
            week_goal: default_week_goal(),
            holiday_duration: default_holiday_duration(),
            offsets: Offset { entry: 0, exit: 0 },
        };
        unsafe {
            HOLIDAY_DURATION = settings.holiday_duration;
        }
        settings
    }
}

impl<'a> Savable<'a, Settings> for Settings {}

impl Settings {
    pub fn save(&self) {
        let mut file = match File::create("./data/settings.json") {
            Err(why) => panic!("couldn't create settings.json: {}", why.description()),
            Ok(file) => file,
        };
        unsafe {
            HOLIDAY_DURATION = self.holiday_duration;
        }
        file.write_all(self.to_json().as_bytes())
            .expect("Unable to write data");
        file.write("\n".as_bytes()).expect("Unable to write \\n");
    }

    pub fn load() -> Option<Settings> {
        match File::open("./data/settings.json") {
            Err(_) => None,
            Ok(mut file) => {
                let mut json = String::new();
                file.read_to_string(&mut json)
                    .expect("Failed to read settings.json");
                let settings = Settings::from_json(&json)
                                .expect("Settings format has changed, please backup `data/settings.json` and delete it.");
                unsafe {
                    HOLIDAY_DURATION = settings.holiday_duration;
                }
                Some(settings)
            }
        }
    }

    pub fn get_default_day_settings_for(&self, day: &NaiveDate) -> SettingsDay {
        match day.weekday() {
            Weekday::Sat | Weekday::Sun => {
                let mut d = SettingsDay::default();
                d.weekday = day.weekday();
                d
            }
            _ => match self.week_sched.sched.binary_search_by(|flex_day| {
                let num_left = flex_day.weekday.number_from_monday();
                let num_right = day.weekday().number_from_monday();
                if flex_day.weekday == day.weekday() {
                    Ordering::Equal
                } else {
                    if num_left > num_right {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                }
            }) {
                Ok(idx) => self.week_sched.sched[idx],
                Err(_) => panic!("couldn't find {:?} in week sched", day.weekday()),
            },
        }
    }
}

/*
** TESTS
*/

#[cfg(test)]
mod tests {
    use super::*;

    fn expected_test_json() -> &'static str {
        r#"{
  "week_sched": {
    "sched": [
      {
        "weekday": "Mon",
        "start": "09:10:00",
        "end": "17:10:00",
        "pause": 30
      },
      {
        "weekday": "Tue",
        "start": "09:10:00",
        "end": "17:10:00",
        "pause": 30
      },
      {
        "weekday": "Wed",
        "start": "09:10:00",
        "end": "17:10:00",
        "pause": 30
      },
      {
        "weekday": "Thu",
        "start": "09:10:00",
        "end": "17:10:00",
        "pause": 30
      },
      {
        "weekday": "Fri",
        "start": "09:10:00",
        "end": "16:50:00",
        "pause": 30
      }
    ]
  },
  "holidays_per_year": 26.0,
  "week_goal": 2220,
  "holiday_duration": 444,
  "offset": 0
}"#
    }

    #[test]
    fn save_and_load_test() {
        let settings = Settings::default();
        settings.save();
        assert!(File::open("./data/settings.json").is_ok());
        let loaded = Settings::load();
        assert_eq!(loaded, Some(settings));
    }

    #[test]
    fn settings_to_json_test() {
        let settings: Settings = Default::default();
        let serialized = settings.to_json();
        //println!("settings: {}", serialized);
        let expected = expected_test_json();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn settings_from_json_test() {
        let json = expected_test_json();
        let settings = Settings::from_json(json);
        let expected: Settings = Default::default();
        assert_eq!(settings, expected);
    }

    #[test]
    fn get_default_day_settings_for_test() {
        let settings: Settings = Default::default();

        let cur_date = NaiveDate::from_ymd(2017, 05, 05);
        let expected = SettingsDay {
            weekday: Weekday::Fri,
            start: NaiveTime::from_hms(9, 10, 00),
            end: NaiveTime::from_hms(16, 50, 00),
            pause: 30,
        };
        assert_eq!(settings.get_default_day_settings_for(&cur_date), expected);
    }
}
