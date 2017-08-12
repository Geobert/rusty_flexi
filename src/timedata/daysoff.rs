use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use settings::Settings;
use savable::Savable;

#[derive(Serialize, Deserialize, Clone)]
pub struct DaysOff {
    year: i32,
    pub holidays_left: f32,
    pub sick_days_taken: f32,
}

impl<'a> Savable<'a, DaysOff> for DaysOff {}

impl DaysOff {
    pub fn new(year: i32, settings: &Settings) -> DaysOff {
        DaysOff {
            year: year,
            holidays_left: settings.holidays_per_year,
            sick_days_taken: 0.0
        }
    }

    pub fn filename(year: i32) -> String {
        format!("./data/{}_daysoff.json", year)
    }

    pub fn save(&self) {
        let mut file = match File::create(DaysOff::filename(self.year)) {
            Err(why) => panic!("couldn't create {}: {}", DaysOff::filename(self.year),
                               why.description()),
            Ok(file) => file,
        };

        file.write_all(self.to_json().as_bytes()).expect("Unable to write data");
        file.write("\n".as_bytes()).expect("Unable to write \\n");
    }

    pub fn load(year: i32, settings: &Settings) -> DaysOff {
        match File::open(DaysOff::filename(year)) {
            Err(_) => DaysOff::new(year, &settings),
            Ok(mut file) => {
                let mut json = String::new();
                file.read_to_string(&mut json)
                    .expect(&format!("Failed to read {}", DaysOff::filename(year)));
                DaysOff::from_json(&json)
            }
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn load_save_load() {
        let s = Settings::default();
        let mut d1 = DaysOff::new(2017, &s);
        assert_eq!(d1.year, 2017);
        assert_eq!(d1.sick_days_taken, 0.0);
        assert_eq!(d1.holidays_left, 26.0);
        d1.holidays_left = 15.0;
        d1.sick_days_taken = 2.0;
        d1.save();
        let d2 = DaysOff::load(2017, &s);
        assert_eq!(d2.year, 2017);
        assert_eq!(d2.holidays_left, 15.0);
        assert_eq!(d2.sick_days_taken, 2.0);
    }
}