use crate::savable::Savable;
use crate::settings::Settings;
use crate::timedata::{DayStatus, FlexDay, FlexWeek, NaiveDateIter, SickDays};
use chrono::{Datelike, NaiveDate, Weekday};
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter, Result};
use std::fs::File;
use std::io::prelude::*;

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct FlexMonth {
    pub weeks: Vec<FlexWeek>,
    pub year: i32,
    pub month: u32,
    pub one_week_goal: i64,
    pub balance: i64, // TODO switch i64 to Duration when chrono supports Serialize/Deserialize
}

impl Display for FlexMonth {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for w in &self.weeks {
            writeln!(f, "{}", w).expect("Failed to write FlexMonth to Display");
        }
        write!(f, "")
    }
}

pub fn next_month(year: i32, month: u32) -> (i32, u32) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

pub fn prev_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

fn find_next_monday(day: NaiveDate) -> NaiveDate {
    match day.weekday() {
        Weekday::Mon => day,
        _ => find_next_monday(day.succ()),
    }
}

fn find_prec_monday(day: NaiveDate) -> NaiveDate {
    match day.weekday() {
        Weekday::Mon => day,
        _ => find_prec_monday(day.pred()),
    }
}

pub fn find_first_monday_of_grid(year: i32, month: u32) -> NaiveDate {
    //println!("find_first_monday_of_grid: year={}, month={}", year, month);
    let first_day = NaiveDate::from_ymd(year, month, 1);
    match first_day.weekday() {
        Weekday::Mon => first_day,
        Weekday::Sat | Weekday::Sun => find_next_monday(first_day),
        _ => find_prec_monday(first_day),
    }
}

pub fn find_last_sunday_for(year: i32, month: u32) -> NaiveDate {
    let (y, m) = next_month(year, month);
    let first_day_next_month = NaiveDate::from_ymd(y, m, 1);
    match first_day_next_month.weekday() {
        Weekday::Sun => first_day_next_month,
        _ => find_first_monday_of_grid(first_day_next_month.year(), first_day_next_month.month())
            .pred(),
    }
}

impl<'a> Savable<'a, FlexMonth> for FlexMonth {}

impl FlexMonth {
    pub fn new(year: i32, month: u32, settings: &Settings) -> FlexMonth {
        let first_day = find_first_monday_of_grid(year, month);
        let last_sunday = find_last_sunday_for(year, month);
        let range = NaiveDateIter::new(first_day, last_sunday);
        let mut weeks: Vec<FlexWeek> = Vec::new();
        let mut week: [FlexDay; 7] = [FlexDay::default(); 7];
        let mut count = 0;
        for d in range {
            week[count % 7] = FlexDay::new(d, settings);
            count += 1;
            if count % 7 == 0 {
                weeks.push(FlexWeek::new(week));
            }
        }
        let balance = weeks.iter().fold(0, |acc, &w| acc + w.total_minutes())
            - settings.week_goal * (weeks.len() as i64);
        FlexMonth {
            weeks: weeks,
            year: year,
            month: month,
            one_week_goal: settings.week_goal,
            balance: balance,
        }
    }

    fn filename(year: i32, month: u32) -> String {
        format!("./data/{}_{:02}.json", year, month)
    }

    pub fn save(&self) {
        let mut file = match File::create(FlexMonth::filename(self.year, self.month)) {
            Err(why) => panic!("couldn't create file: {}", why.description()),
            Ok(file) => file,
        };

        file.write_all(self.to_json().as_bytes())
            .expect("Unable to write data");
        file.write("\n".as_bytes()).expect("Unable to write");
    }

    /// return FlexMonth and if it was loaded from json or not
    pub fn load_with_flag(year: i32, month: u32, settings: &Settings) -> (FlexMonth, bool) {
        match File::open(FlexMonth::filename(year, month)) {
            Err(_) => (FlexMonth::new(year, month, &settings), false),
            Ok(mut file) => {
                let mut json = String::new();
                file.read_to_string(&mut json).expect(&format!(
                    "Failed to read file: {}",
                    FlexMonth::filename(year, month)
                ));
                (
                    FlexMonth::from_json(&json).expect(&format!(
                        "Failed to deserialized {}",
                        FlexMonth::filename(year, month)
                    )),
                    true,
                )
            }
        }
    }

    pub fn load(year: i32, month: u32, settings: &Settings) -> FlexMonth {
        let (month, from_json) = FlexMonth::load_with_flag(year, month, &settings);
        // generate Xmas holidays if needed
        if !from_json {
            // newly created month
            match month.month {
                1 => {
                    let mut january = month;
                    // auto set holiday as we always have 2 days of holidays in january
                    let mut week_to_edit = january.weeks[0].clone();
                    week_to_edit[0].status = DayStatus::Holiday;
                    week_to_edit[1].status = DayStatus::Holiday;
                    january.weeks[0] = week_to_edit;
                    january.save();
                    january
                }
                12 => {
                    let mut december = month;
                    // auto set holiday as we always have 5 days of holidays in december
                    let nb_weeks = december.weeks.len();
                    let mut week_to_edit = december.weeks[nb_weeks - 1].clone();
                    for day in week_to_edit.days.iter_mut() {
                        if day.status == DayStatus::Worked {
                            day.status = DayStatus::Holiday;
                        }
                    }
                    december.weeks[nb_weeks - 1] = week_to_edit;
                    december.save();
                    december
                }
                _ => month,
            }
        } else {
            month
        }
    }

    pub fn load_with_file(path: String) -> FlexMonth {
        let mut file = File::open(&path).unwrap();
        let mut json = String::new();
        file.read_to_string(&mut json)
            .expect(&format!("Failed to read file {}", path));
        FlexMonth::from_json(&json).expect(&format!("Failed to deserialized  {}", path))
    }

    pub fn get_week_with_day(&self, d: NaiveDate) -> Option<(&FlexDay, &FlexWeek, i32)> {
        let mut week_number = 1;
        for w in &self.weeks {
            if let Some(day) = w.days.iter().find(|&&day| {
                if let Some(date) = day.date {
                    date == d
                } else {
                    false
                }
            }) {
                return Some((&day, &w, week_number));
            }
            week_number += 1;
        }
        None
    }

    pub fn total_minute(&self) -> i64 {
        self.weeks.iter().fold(0, |acc, &w| acc + w.total_minutes())
    }

    pub fn update_balance(&mut self) {
        self.balance = self.total_minute() - self.one_week_goal * (self.weeks.len() as i64);
    }

    pub fn update_day(&mut self, d: FlexDay) -> Option<FlexWeek> {
        for w in &mut self.weeks {
            for i in 0..w.days.len() {
                if w.days[i].date == d.date {
                    w.days[i] = d;
                    return Some(w.clone());
                }
            }
        }
        None
    }

    pub fn get_sick_days(&self) -> SickDays {
        self.weeks
            .iter()
            .flat_map(|w| {
                w.days
                    .to_vec()
                    .into_iter()
                    .filter(|d| d.status == DayStatus::Sick)
                    .map(|d| d.date.unwrap())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timedata;

    #[test]
    fn get_week_with_day_test() {
        let settings: Settings = Default::default();
        let m = FlexMonth::new(2017, 05, &settings);
        let w = m
            .get_week_with_day(NaiveDate::from_ymd(2017, 05, 10))
            .unwrap();
        assert_eq!(w.1.days[0].date.unwrap().day(), 8);
    }

    #[test]
    fn find_next_monday_test() {
        let mut day = NaiveDate::from_ymd(2017, 04, 01);
        let mut first_monday = find_next_monday(day);
        assert_eq!(first_monday, NaiveDate::from_ymd(2017, 04, 03));

        day = NaiveDate::from_ymd(2017, 05, 01);
        first_monday = find_next_monday(day);
        assert_eq!(first_monday, NaiveDate::from_ymd(2017, 05, 01));

        day = NaiveDate::from_ymd(2017, 03, 01);
        first_monday = find_next_monday(day);
        assert_eq!(first_monday, NaiveDate::from_ymd(2017, 03, 06));

        day = NaiveDate::from_ymd(2016, 12, 01);
        first_monday = find_next_monday(day);
        assert_eq!(first_monday, NaiveDate::from_ymd(2016, 12, 05));
    }

    #[test]
    fn find_prec_monday_test() {
        let mut day = NaiveDate::from_ymd(2017, 04, 01);
        let mut monday = find_prec_monday(day);
        assert_eq!(monday, NaiveDate::from_ymd(2017, 03, 27));

        day = NaiveDate::from_ymd(2017, 05, 01);
        monday = find_prec_monday(day);
        assert_eq!(monday, NaiveDate::from_ymd(2017, 05, 01));

        day = NaiveDate::from_ymd(2017, 03, 01);
        monday = find_prec_monday(day);
        assert_eq!(monday, NaiveDate::from_ymd(2017, 02, 27));

        day = NaiveDate::from_ymd(2016, 12, 01);
        monday = find_prec_monday(day);
        assert_eq!(monday, NaiveDate::from_ymd(2016, 11, 28));
    }

    #[test]
    fn find_last_sunday_test() {
        let mut sunday = find_last_sunday_for(2017, 04);
        assert_eq!(sunday, NaiveDate::from_ymd(2017, 04, 30));

        sunday = find_last_sunday_for(2017, 05);
        assert_eq!(sunday, NaiveDate::from_ymd(2017, 05, 28));

        sunday = find_last_sunday_for(2017, 03);
        assert_eq!(sunday, NaiveDate::from_ymd(2017, 04, 02));

        sunday = find_last_sunday_for(2016, 12);
        assert_eq!(sunday, NaiveDate::from_ymd(2017, 01, 01));
    }

    #[test]
    fn create_month_test() {
        let settings: Settings = Default::default();
        let mut month = FlexMonth::new(2017, 05, &settings);

        assert_eq!(month.weeks.len(), 4);
        assert_eq!(
            month.weeks[0].days[0].date,
            Some(NaiveDate::from_ymd(2017, 05, 01))
        );
        assert_eq!(
            month.weeks[3].days[6].date,
            Some(NaiveDate::from_ymd(2017, 05, 28))
        );

        month = FlexMonth::new(2017, 02, &settings);
        assert_eq!(month.weeks.len(), 4);
        assert_eq!(
            month.weeks[0].days[0].date,
            Some(NaiveDate::from_ymd(2017, 01, 30))
        );
        assert_eq!(
            month.weeks[3].days[6].date,
            Some(NaiveDate::from_ymd(2017, 02, 26))
        );

        month = FlexMonth::new(2017, 04, &settings);
        assert_eq!(month.weeks.len(), 4);
        assert_eq!(
            month.weeks[0].days[0].date,
            Some(NaiveDate::from_ymd(2017, 04, 03))
        );
        assert_eq!(
            month.weeks[3].days[6].date,
            Some(NaiveDate::from_ymd(2017, 04, 30))
        );

        month = FlexMonth::new(2017, 01, &settings);
        assert_eq!(month.weeks.len(), 4);
        assert_eq!(
            month.weeks[0].days[0].date,
            Some(NaiveDate::from_ymd(2017, 01, 02))
        );
        assert_eq!(
            month.weeks[3].days[6].date,
            Some(NaiveDate::from_ymd(2017, 01, 29))
        );

        month = FlexMonth::new(2016, 11, &settings);
        assert_eq!(month.weeks.len(), 4);
        assert_eq!(
            month.weeks[0].days[0].date,
            Some(NaiveDate::from_ymd(2016, 10, 31))
        );
        assert_eq!(
            month.weeks[3].days[6].date,
            Some(NaiveDate::from_ymd(2016, 11, 27))
        );

        month = FlexMonth::new(2016, 12, &settings);
        assert_eq!(month.weeks.len(), 5);
        assert_eq!(
            month.weeks[0].days[0].date,
            Some(NaiveDate::from_ymd(2016, 11, 28))
        );
        assert_eq!(
            month.weeks[4].days[6].date,
            Some(NaiveDate::from_ymd(2017, 01, 01))
        );
    }

    #[test]
    fn save_load_test() {
        timedata::create_data_dir();
        let settings: Settings = Default::default();
        let m = FlexMonth::new(2017, 05, &settings);
        m.save();
        let loaded = FlexMonth::load(2017, 05, &settings);
        assert_eq!(m, loaded);
    }
}
