use timedata::{FlexWeek, NaiveDateIter, FlexDay};
use chrono::{NaiveDate, Weekday, Datelike};
use settings::Settings;
use std::fs::File;

use std::io::prelude::*;
use std::error::Error;
use savable::Savable;

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct FlexMonth {
    pub weeks: Vec<FlexWeek>,
    pub year: i32,
    pub month: u32,
    pub hours: i64,
    pub balance: i64
    // TODO switch i64 to Duration when chrono supports Serialize/Deserialize
}

fn find_next_monday(day: NaiveDate) -> NaiveDate {
    match day.weekday() {
        Weekday::Mon => day,
        _ => find_next_monday(day.succ())
    }
}

fn find_prec_monday(day: NaiveDate) -> NaiveDate {
    match day.weekday() {
        Weekday::Mon => day,
        _ => find_prec_monday(day.pred())
    }
}

fn find_first_monday_of_grid(year: i32, month: u32) -> NaiveDate {
    let first_day = NaiveDate::from_ymd(year, month, 1);
    match first_day.weekday() {
        Weekday::Mon => first_day,
        Weekday::Sat | Weekday::Sun => find_next_monday(first_day),
        _ => find_prec_monday(first_day)
    }
}

fn find_last_sunday_for(year: i32, month: u32) -> NaiveDate {
    let (y, m) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    let first_day_next_month = NaiveDate::from_ymd(y, m, 1);
    match first_day_next_month.weekday() {
        Weekday::Sun => first_day_next_month,
        _ => find_first_monday_of_grid(first_day_next_month.year(),
                                       first_day_next_month.month()).pred()
    }
}

impl<'a> Savable<'a, FlexMonth> for FlexMonth {}

impl FlexMonth {
    pub fn new(year: i32, month: u32, settings: &Settings) -> FlexMonth {
        let first_day = find_first_monday_of_grid(year, month);
        let last_sunday = find_last_sunday_for(year, month);
        let range = NaiveDateIter::new(first_day, last_sunday);
        let mut weeks: Vec<FlexWeek> = Vec::new();
        let mut week: [FlexDay; 7] = [Default::default(); 7];
        let mut count = 0;
        for d in range {
            week[count % 7] = FlexDay::new(d, settings);
            count += 1;
            if count % 7 == 0 {
                weeks.push(FlexWeek::new(week));
            }
        }
        let balance = weeks.iter().fold(0, |acc, &w| acc + w.hours) -
            settings.week_goal * (weeks.len() as i64);
        FlexMonth {
            weeks: weeks,
            year: year,
            month: month,
            hours: settings.week_goal,
            balance: balance
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

        file.write_all(self.to_json().as_bytes()).expect("Unable to write data");
        file.write("\n".as_bytes()).expect("Unable to write");
    }

    pub fn load(year: i32, month: u32) -> FlexMonth {
        match File::open(FlexMonth::filename(year, month)) {
            Err(_) => Default::default(),
            Ok(mut file) => {
                let mut json = String::new();
                file.read_to_string(&mut json).expect("Failed to read file");
                FlexMonth::from_json(&json)
            }
        }
    }


}

#[cfg(test)]
mod tests
{
    use super::*;
    use timedata;

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
        assert_eq!(month.weeks[0].days[0].date, Some(NaiveDate::from_ymd(2017, 05, 01)));
        assert_eq!(month.weeks[3].days[6].date, Some(NaiveDate::from_ymd(2017, 05, 28)));

        month = FlexMonth::new(2017, 02, &settings);
        assert_eq!(month.weeks.len(), 4);
        assert_eq!(month.weeks[0].days[0].date, Some(NaiveDate::from_ymd(2017, 01, 30)));
        assert_eq!(month.weeks[3].days[6].date, Some(NaiveDate::from_ymd(2017, 02, 26)));

        month = FlexMonth::new(2017, 04, &settings);
        assert_eq!(month.weeks.len(), 4);
        assert_eq!(month.weeks[0].days[0].date, Some(NaiveDate::from_ymd(2017, 04, 03)));
        assert_eq!(month.weeks[3].days[6].date, Some(NaiveDate::from_ymd(2017, 04, 30)));

        month = FlexMonth::new(2017, 01, &settings);
        assert_eq!(month.weeks.len(), 4);
        assert_eq!(month.weeks[0].days[0].date, Some(NaiveDate::from_ymd(2017, 01, 02)));
        assert_eq!(month.weeks[3].days[6].date, Some(NaiveDate::from_ymd(2017, 01, 29)));

        month = FlexMonth::new(2016, 11, &settings);
        assert_eq!(month.weeks.len(), 4);
        assert_eq!(month.weeks[0].days[0].date, Some(NaiveDate::from_ymd(2016, 10, 31)));
        assert_eq!(month.weeks[3].days[6].date, Some(NaiveDate::from_ymd(2016, 11, 27)));

        month = FlexMonth::new(2016, 12, &settings);
        assert_eq!(month.weeks.len(), 5);
        assert_eq!(month.weeks[0].days[0].date, Some(NaiveDate::from_ymd(2016, 11, 28)));
        assert_eq!(month.weeks[4].days[6].date, Some(NaiveDate::from_ymd(2017, 01, 01)));
    }
    
    #[test]
    fn save_load_test() {
        timedata::create_data_dir();
        let settings: Settings = Default::default();
        let m = FlexMonth::new(2017, 05, &settings);
        m.save();
        let loaded = FlexMonth::load(2017, 05);
        assert_eq!(m, loaded);
    }
}