use chrono::Duration;
use timedata::FlexDay;
use timedata::NaiveDateIter;

#[derive(Copy, Clone)]
pub struct FlexWeek
{
    pub days: [FlexDay; 7],
    pub hours: Duration
}


impl FlexWeek {
    pub fn new(days: [FlexDay; 7]) -> FlexWeek {
        // TODO get default values from settings
        FlexWeek { days: days, hours: Duration::hours(0) }
    }
}