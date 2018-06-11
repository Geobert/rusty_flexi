use chrono;
use chrono::naive::NaiveDate;
use chrono::Datelike;
use failure::Error;
use glob::glob;
use savable::Savable;
use settings::Settings;
use std::fs::File;
use std::io::prelude::*;
use timedata::*;

pub type SickDays = Vec<NaiveDate>;

#[derive(Serialize, Deserialize, Clone)]
pub struct DaysOff {
    year: i32,
    pub holidays_left: f32,
    #[serde(skip)] // sick days are in another standalone file
    pub sick_days: SickDays,
}

impl<'a> Savable<'a, DaysOff> for DaysOff {}
impl<'a> Savable<'a, SickDays> for SickDays {}

impl DaysOff {
    pub fn new(year: i32, settings: &Settings) -> DaysOff {
        DaysOff {
            year: year,
            holidays_left: settings.holidays_per_year,
            sick_days: SickDays::default(),
        }
    }

    pub fn filename(year: i32) -> String {
        format!("./data/{}_daysoff.json", year)
    }

    pub fn save(&self) -> Result<(), Error> {
        let mut file = File::create(DaysOff::filename(self.year))?;

        file.write_all(self.to_json().as_bytes())?;
        file.write("\n".as_bytes())?;

        let mut file = File::create("./data/sickdays.json")?;
        file.write_all(self.sick_days.to_json().as_bytes())?;
        file.write("\n".as_bytes())?;
        Ok(())
    }

    // TODO change to Result
    pub fn load(year: i32, settings: &Settings) -> DaysOff {
        let mut daysoff = match File::open(DaysOff::filename(year)) {
            Err(_) => DaysOff::new(year, &settings),
            Ok(mut file) => {
                let mut json = String::new();
                file.read_to_string(&mut json)
                    .expect(&format!("Failed to read {}", DaysOff::filename(year)));
                DaysOff::from_json(&json).expect(&format!(
                    "Failed to deserialized {}",
                    DaysOff::filename(year)
                ))
            }
        };

        let mut need_save = false;
        daysoff.sick_days = match File::open("./data/sickdays.json") {
            Err(_) => {
                need_save = true;
                DaysOff::rebuild_sick_days().unwrap() // TODO propagate error
            }
            Ok(mut file) => {
                let mut json = String::new();
                file.read_to_string(&mut json)
                    .expect(&format!("Failed to read sickdays.json"));
                SickDays::from_json(&json).expect(&format!("Failed to deserialized sickdays.json"))
            }
        };
        if need_save {
            daysoff.save().unwrap(); // TODO
        }
        daysoff
    }

    fn rebuild_sick_days() -> Result<SickDays, Error> {
        Ok(glob("./data/[0-9][0-9][0-9][0-9]_[0-9][0-9].json")?
            .flat_map(|path| {
                FlexMonth::load_with_file(path.unwrap().to_string_lossy().to_string())
                    .get_sick_days()
            })
            .collect())
    }

    pub fn update_days_off(&mut self, old_status: DayStatus, day: FlexDay) {
        let new_status = day.status;
        if old_status != new_status {
            match old_status {
                DayStatus::Worked => match new_status {
                    DayStatus::Holiday => self.holidays_left -= 1.0,
                    DayStatus::Half => self.holidays_left -= 0.5,
                    DayStatus::Sick => self.add_sick_day(day),
                    DayStatus::Weekend | DayStatus::Worked => {}
                },
                DayStatus::Holiday => match new_status {
                    DayStatus::Worked | DayStatus::Weekend => self.holidays_left += 1.0,
                    DayStatus::Half => self.holidays_left += 0.5,
                    DayStatus::Sick => {
                        self.holidays_left += 1.0;
                        self.add_sick_day(day);
                    }
                    DayStatus::Holiday => {}
                },
                DayStatus::Half => match new_status {
                    DayStatus::Worked | DayStatus::Weekend => self.holidays_left += 0.5,
                    DayStatus::Holiday => self.holidays_left -= 0.5,
                    DayStatus::Sick => {
                        self.holidays_left += 0.5;
                        self.add_sick_day(day);
                    }
                    DayStatus::Half => {}
                },
                DayStatus::Sick => {
                    self.remove_sick_day(day);
                    match new_status {
                        DayStatus::Half => self.holidays_left -= 0.5,
                        DayStatus::Holiday => self.holidays_left -= 1.0,
                        DayStatus::Sick => {}
                        DayStatus::Worked | DayStatus::Weekend => {}
                    }
                }
                DayStatus::Weekend => {}
            }
        }
    }

    fn add_sick_day(&mut self, d: FlexDay) {
        let date = d.date.expect("sick day should have date");
        if let Err(insert_idx) = self.sick_days.binary_search(&date) {
            self.sick_days.insert(insert_idx, date);
        }
        self.roll_sick_days();
    }

    fn remove_sick_day(&mut self, d: FlexDay) {
        let date = d.date.expect("sick day should have date");
        if let Ok(index) = self.sick_days.binary_search(&date) {
            self.sick_days.remove(index);
        }
        self.roll_sick_days();
    }

    pub fn roll_sick_days(&mut self) {
        let today = chrono::Local::today().naive_local();
        let limit = NaiveDate::from_ymd(today.year() - 1, today.month(), 1);
        self.sick_days.retain(|&date| date > limit);
    }

    pub fn sick_days_taken(&self) -> f32 {
        self.sick_days.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_save_load() {
        let s = Settings::default();
        let mut d1 = DaysOff::new(2017, &s);
        assert_eq!(d1.year, 2017);
        assert_eq!(d1.sick_days_taken(), 0.0);
        assert_eq!(d1.holidays_left, 26.0);
        d1.holidays_left = 15.0;
        d1.sick_days.push(FlexDay::default());
        d1.save().unwrap();
        let d2 = DaysOff::load(2017, &s);
        assert_eq!(d2.year, 2017);
        assert_eq!(d2.holidays_left, 15.0);
        assert_eq!(d2.sick_days_taken(), 1.0);
    }

    #[test]
    fn sick_day_test() {
        let s = Settings::default();
        let mut d1 = DaysOff::new(2017, &s);
        assert_eq!(d1.sick_days_taken(), 0.0);
        let today = chrono::Local::today().naive_local();
        let day = FlexDay::new(today, &s);
        d1.add_sick_day(day);
        assert_eq!(d1.sick_days_taken(), 1.0);

        // adding the same day is not authorised
        d1.add_sick_day(day);
        assert_eq!(d1.sick_days_taken(), 1.0);

        d1.save().unwrap();
        let d2 = DaysOff::load(2017, &s);
        assert_eq!(d2.sick_days_taken(), 1.0);

        // sick days are in a stand alone file but managed by DaysOff struct
        let mut d2 = DaysOff::load(2018, &s);
        assert_eq!(d2.sick_days_taken(), 1.0);

        // adding a day more than 12 months old should be removed by roll_sick_days
        let limit = NaiveDate::from_ymd(today.year() - 1, today.month(), 1).pred();
        let day = FlexDay::new(limit, &s);
        d2.add_sick_day(day);
        assert_eq!(d2.sick_days_taken(), 1.0);

        let day = FlexDay::new(today, &s);
        d2.remove_sick_day(day);
        assert_eq!(d2.sick_days_taken(), 0.0);
        d2.remove_sick_day(day);
        assert_eq!(d2.sick_days_taken(), 0.0);
    }
}
