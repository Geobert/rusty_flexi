use timedata::{FlexWeek, NaiveDateIter, FlexDay};
use chrono::{Duration, NaiveDate, Weekday, Datelike};
use settings::Settings;

pub struct FlexMonth {
    pub weeks: Vec<FlexWeek>,
    pub hours: Duration,
    pub balance: Duration
}

fn find_first_monday(year: i32, month: u32, day: u32) -> NaiveDate {
    let first_day = NaiveDate::from_ymd(year, month, day);
    if first_day.weekday() != Weekday::Mon {
        find_first_monday(year, month, day + 1)
    } else {
        first_day
    }
}

#[test]
fn find_first_monday_test() {
    let mut first_monday = find_first_monday(2017, 04, 01);
    assert_eq!(first_monday.day(), 3);

    first_monday = find_first_monday(2017, 05, 01);
    assert_eq!(first_monday.day(), 1);

    first_monday = find_first_monday(2017, 03, 01);
    assert_eq!(first_monday.day(), 6);

    let last_sunday = find_first_monday(2017, 04, 01).pred();
    assert_eq!(last_sunday.weekday(), Weekday::Sun);
}

impl FlexMonth {
    pub fn new(year: i32, month: u32, settings: Settings) -> FlexMonth {
        let first_day = NaiveDate::from_ymd(year, month, 1);
        let first_monday = find_first_monday(year, month, 1);
        let last_sunday = find_first_monday(year, month + 1, 1).pred();
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