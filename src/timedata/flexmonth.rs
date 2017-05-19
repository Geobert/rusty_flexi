use timedata::{FlexWeek, NaiveDateIter, FlexDay};
use chrono::{Duration, NaiveDate, Weekday, Datelike};
use settings::Settings;

pub struct FlexMonth {
    pub weeks: Vec<FlexWeek>,
    pub hours: Duration,
    pub balance: Duration
}

fn find_first_monday(day: NaiveDate) -> NaiveDate {
    match day.weekday() {
        Weekday::Mon => day,
        _ => find_first_monday(day.succ())
    }
}

fn find_first_monday_for(year: i32, month: u32) -> NaiveDate {
    let first_day = NaiveDate::from_ymd(year, month, 1);
    find_first_monday(first_day)
}

fn find_last_monday(day: NaiveDate) -> NaiveDate {
        match day.weekday() {
            Weekday::Mon => day,
            _ => find_last_monday(day.pred())
        }
    }

fn find_last_monday_for(year: i32, month: u32) -> NaiveDate {
    let last_day = NaiveDate::from_ymd(year, month + 1, 1).pred();
    find_last_monday(last_day)
}

#[test]
fn find_first_monday_test() {
    let mut first_monday = find_first_monday_for(2017, 04);
    assert_eq!(first_monday.day(), 3);

    first_monday = find_first_monday_for(2017, 05);
    assert_eq!(first_monday.day(), 1);

    first_monday = find_first_monday_for(2017, 03);
    assert_eq!(first_monday.day(), 6);

    let last_sunday = find_first_monday_for(2017, 04).pred();
    assert_eq!(last_sunday.weekday(), Weekday::Sun);
}

#[test]
fn find_last_monday_test() {
    let mut last_monday = find_last_monday_for(2017, 04);
    assert_eq!(last_monday.day(), 24);

    last_monday = find_last_monday_for(2017, 05);
    assert_eq!(last_monday.day(), 29);

    last_monday = find_last_monday_for(2017, 07);
    assert_eq!(last_monday.day(), 31);

    last_monday = find_last_monday_for(2017, 02);
    assert_eq!(last_monday.day(), 27);
}

impl FlexMonth {
    pub fn new(year: i32, month: u32, settings: Settings) -> FlexMonth {
        let first_day = NaiveDate::from_ymd(year, month, 1);
        let first_monday = match first_day.weekday() {
            Weekday::Sat | Weekday::Sun => find_first_monday(first_day),
            _ => find_last_monday(first_day.pred())
        };
        
        let last_sunday = find_first_monday_for(year, month + 1).pred();
        let range = NaiveDateIter::new(first_monday, last_sunday);
        let mut weeks: Vec<FlexWeek> = Vec::new();
        let mut count = 0;
        for d in range {
            let weekdaynum = count % 7;
            if weekdaynum == 0 {
                weeks.push(FlexWeek::new([FlexDay::new(); 7]));
            }
            count += 1;
        }
        FlexMonth { weeks: weeks, hours: Duration::hours(0), balance: Duration::hours(0) }
    }
}

#[test]
fn create_month_test() {
    let settings = Settings::load();
    let mut month = FlexMonth::new(2017, 05, settings);
    assert_eq!(month.weeks.len, 5);
    assert_eq!(month.weeks[0].days[0].date, NaiveDate::from_ymd(2017, 05, 01));
    assert_eq!(month.weeks[4].days[6].date , NaiveDate::from_ymd(2017, 06, 04));
}