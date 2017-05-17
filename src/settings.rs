use serde_json;
use timedata::{ FlexDay };
use chrono::{ Duration, Weekday, NaiveTime };
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct WeekSchedule {
    default: FlexDay,
    exceptions: Vec<FlexDay>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Holidays {
    year: u32,
    left: u32
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Settings {
    week_sched: WeekSchedule,
    holidays: Holidays,
    week_goal: i64, // TODO switch to Duration when chrono supports Serialize/Deserialize
}

impl Settings {
    pub fn from_json(serialized: String) -> Settings {
        let settings = serde_json::from_str(&serialized).unwrap();
        settings
    }

    pub fn to_json(&self) -> String {
        let serialized = serde_json::to_string(&self).unwrap();
        serialized
    }

    pub fn save(&self) {
        let mut file = match File::create("settings.json") {
            Err(why) => panic!("couldn't create settings.json: {}", why.description()),
            Ok(file) => file,
        };
        
        file.write_all(self.to_json().as_bytes()).expect("Unable to write data");
    }

    pub fn load() -> Settings {
        match File::open("settings.json") {
            Err(_) => Default::default(),
            Ok(mut file) => {
                let mut json = String::new();
                file.read_to_string(&mut json).expect("Failed to read settings.json");
                Settings::from_json(json)
            }
        }
    }
}

/*
** TESTS
*/

fn build_test_settings() -> Settings {
    let ex_day = FlexDay { day: Some(Weekday::Fri), start: NaiveTime::from_hms(9, 10, 00), end: NaiveTime::from_hms(16, 50, 00), pause: Duration::minutes(30).num_minutes(), ..Default::default() };
    let def_day = FlexDay { day: None, start: NaiveTime::from_hms(9, 10, 00), end: NaiveTime::from_hms(17, 10, 00), pause: Duration::minutes(30).num_minutes(), ..Default::default() };

    let settings = Settings {
        week_sched: WeekSchedule {
            default: def_day,
            exceptions: vec![ex_day]
        },
        holidays: Holidays { year: 26, left: 26 },
        week_goal: Duration::hours(148).num_minutes()
    };
    settings
}

fn expected_test_json() -> String {
    "{\"week_sched\":{\"default\":{\"day\":null,\"start\":\"09:10:00\",\"end\":\"17:10:00\",\"pause\":30,\"status\":\"Worked\"},\"exceptions\":[{\"day\":\"Fri\",\"start\":\"09:10:00\",\"end\":\"16:50:00\",\"pause\":30,\"status\":\"Worked\"}]},\"holidays\":{\"year\":26,\"left\":26},\"week_goal\":8880}".to_string()
}

#[test]
fn save_and_load_test() {
    let settings = build_test_settings();
    settings.save();
    assert!(File::open("settings.json").is_ok());
    let loaded = Settings::load();
    assert_eq!(loaded, settings);
}

#[test]
fn settings_to_json_test() {
    let settings = build_test_settings();
    let serialized = settings.to_json();
    let expected = expected_test_json();
    assert_eq!(serialized, expected);
}

#[test]
fn settings_from_json_test() {
    let json = expected_test_json();
    let settings = Settings::from_json(json);
    let expected = build_test_settings();
    assert_eq!(settings, expected);
}