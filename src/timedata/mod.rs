pub use self::flexday::FlexDay;
pub use self::flexday::HOLIDAY_DURATION;
pub use self::flexweek::FlexWeek;
pub use self::flexmonth::FlexMonth;
pub use self::naivedate_iterator::NaiveDateIter;
pub use self::flexday::DayStatus;

mod flexday;
mod flexweek;
mod flexmonth;
mod naivedate_iterator;

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