use chrono::Weekday;
use timedata::FlexDay;
use std::iter::Iterator;
use std::default::Default;

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct FlexWeek
{
    pub days: [FlexDay; 7],
    pub hours: i64
}


impl FlexWeek {
    pub fn new(days: [FlexDay; 7]) -> FlexWeek {
        FlexWeek { days: days, hours: FlexWeek::total_minutes_of(days) }
    }

    fn total_minutes_of(days: [FlexDay; 7]) -> i64 {
        days.iter().fold(0, |acc, &day| acc + day.total_minutes())
    }

    pub fn total_minutes(&self) -> i64 {
        FlexWeek::total_minutes_of(self.days)
    }

    pub fn update(&mut self) {
        self.hours = self.total_minutes();
    }
}

impl Default for FlexWeek {
    fn default() -> FlexWeek {
        let mut w = FlexWeek {
            days: [Default::default(); 7],
            hours: 0,
        };
        let mut wd = Weekday::Mon;
        for day in &mut(w.days) {
            day.set_weekday(wd);
            wd = wd.succ();
        }
        w
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn total_minutes_test() {
        let mut w = FlexWeek { days: [Default::default(); 7], hours: 0 };
        assert_eq!(w.total_minutes(), (8 * 60 - 30) * 7);

        w = Default::default();
        assert_eq!(w.total_minutes(), (8 * 60 - 30) * 5);
    }
}