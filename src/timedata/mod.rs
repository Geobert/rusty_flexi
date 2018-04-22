pub use self::flexday::FlexDay;
pub use self::flexday::HOLIDAY_DURATION;
pub use self::flexweek::FlexWeek;
pub use self::flexmonth::FlexMonth;
pub use self::flexmonth::find_first_monday_of_grid;
pub use self::flexmonth::find_last_sunday_for;
pub use self::flexmonth::next_month;
pub use self::flexmonth::prev_month;
pub use self::naivedate_iterator::NaiveDateIter;
pub use self::flexday::DayStatus;
pub use self::daysoff::DaysOff;
pub use self::daysoff::SickDays;

mod flexday;
mod flexweek;
mod flexmonth;
mod naivedate_iterator;
mod daysoff;

use std::fs;
use std::path::Path;
use chrono::Weekday;

pub fn create_data_dir() {
    let dir = Path::new("./data");
    if !dir.exists() {
        match fs::create_dir_all(dir) {
            Err(why) => println!("failed to create data dir: {}", why),
            _ => {}
        }
    }
}

pub fn weekday_to_string(wd: Weekday) -> String {
    match wd {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }.to_string()
}

pub fn month_to_string(m: u32) -> &'static str {
    match m {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "???",
    }
}
