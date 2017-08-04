use chrono::{Duration, Weekday, NaiveTime, NaiveDate, Datelike};
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use std::cmp::Ordering;
use savable::Savable;
use timedata::HOLIDAY_DURATION;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct SettingsDay {
    pub weekday: Option<Weekday>,
    pub start: NaiveTime,
    pub end: NaiveTime,
    // TODO switch to Duration when chrono supports Serialize/Deserialize
    pub pause: i64,
}

impl Default for SettingsDay {
    fn default() -> SettingsDay {
        SettingsDay {
            weekday: None,
            start: NaiveTime::from_hms(9, 0, 0),
            end: NaiveTime::from_hms(17, 0, 0),
            pause: Duration::minutes(30).num_minutes(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct WeekSchedule {
    pub default: SettingsDay,
    pub exceptions: Vec<SettingsDay>
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Settings {
    pub week_sched: WeekSchedule,
    pub holidays_per_year: f32,
    pub week_goal: i64,
    pub holiday_duration: i64,
    // TODO switch to Duration when chrono supports Serialize/Deserialize
}

impl Default for Settings {
    fn default() -> Settings {
        let ex_day = SettingsDay {
            start: NaiveTime::from_hms(9, 10, 00),
            end: NaiveTime::from_hms(16, 50, 00),
            pause: 30,
            weekday: Some(Weekday::Fri),
        };

        let def_day = SettingsDay {
            weekday: None,
            start: NaiveTime::from_hms(9, 10, 00),
            end: NaiveTime::from_hms(17, 10, 00),
            pause: 30,
        };

        let settings = Settings {
            week_sched: WeekSchedule {
                default: def_day,
                exceptions: vec![ex_day]
            },
            holidays_per_year: 26.0,
            week_goal: Duration::hours(37).num_minutes(),
            holiday_duration: Duration::hours(37).num_minutes() / 5,
        };
        settings
    }
}

impl<'a> Savable<'a, Settings> for Settings {}

impl Settings {
    pub fn save(&self) {
        let mut file = match File::create("settings.json") {
            Err(why) => panic!("couldn't create settings.json: {}", why.description()),
            Ok(file) => file,
        };

        file.write_all(self.to_json().as_bytes()).expect("Unable to write data");
        file.write("\n".as_bytes()).expect("Unable to write \\n");
    }

    pub fn load() -> Settings {
        let s = match File::open("settings.json") {
            Err(_) => Default::default(),
            Ok(mut file) => {
                let mut json = String::new();
                file.read_to_string(&mut json).expect("Failed to read settings.json");
                Settings::from_json(&json)
            }
        };
        unsafe {
            HOLIDAY_DURATION = s.holiday_duration;
        }
        s
    }

    pub fn get_default_day_settings_for(&self, day: &NaiveDate) -> SettingsDay {
        match day.weekday() {
            Weekday::Sat | Weekday::Sun => {
                let mut d: SettingsDay = Default::default();
                d.weekday = Some(day.weekday());
                d
            }
            _ => match self.week_sched.exceptions.binary_search_by(
                |flex_day| match flex_day.weekday {
                    Some(w) => if w == day.weekday() { Ordering::Equal } else { Ordering::Less },
                    None => Ordering::Less,
                }) {
                Ok(idx) => self.week_sched.exceptions[idx],
                Err(_) => self.week_sched.default,
            }
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
    "default": {
      "weekday": null,
      "start": "09:10:00",
      "end": "17:10:00",
      "pause": 30
    },
    "exceptions": [
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
  "holiday_duration": 444
}"#
    }

    #[test]
    fn save_and_load_test() {
        let settings: Settings = Default::default();
        settings.save();
        assert!(File::open("settings.json").is_ok());
        let loaded = Settings::load();
        assert_eq!(loaded, settings);
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
            weekday: Some(Weekday::Fri),
            start: NaiveTime::from_hms(9, 10, 00),
            end: NaiveTime::from_hms(16, 50, 00),
            pause: 30,
        };
        assert_eq!(settings.get_default_day_settings_for(&cur_date), expected);

        let cur_date = NaiveDate::from_ymd(2017, 05, 04);
        let expected = SettingsDay {
            weekday: None,
            start: NaiveTime::from_hms(9, 10, 00),
            end: NaiveTime::from_hms(17, 10, 00),
            pause: 30,
        };
        assert_eq!(settings.get_default_day_settings_for(&cur_date), expected);
    }
}