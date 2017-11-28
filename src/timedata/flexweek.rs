use chrono::Weekday;
use timedata::FlexDay;
use timedata::DayStatus;
use std::iter::Iterator;
use std::default::Default;
use std::fmt::{Display, Result, Formatter};
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct FlexWeek {
    pub days: [FlexDay; 7],
}

impl Display for FlexWeek {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for d in &self.days {
            if d.status != DayStatus::Weekend {
                writeln!(f, "{}", d).expect("Failed to write FlexWeek to Display");
            }
        }
        let hours = self.total_minutes();
        writeln!(
            f,
            "{:->40} {:02}:{:02}",
            " Total =",
            hours / 60,
            hours - (hours / 60) * 60
        )
    }
}

impl FlexWeek {
    pub fn new(days: [FlexDay; 7]) -> FlexWeek {
        FlexWeek { days: days }
    }

    pub fn total_minutes(&self) -> i64 {
        self.days.iter().fold(
            0,
            |acc, &day| acc + day.total_minutes(),
        )
    }

    pub fn total_str(&self) -> String {
        let hours = self.total_minutes();
        format!("{:02}:{:02}", hours / 60, hours - (hours / 60) * 60)
    }
}

impl Default for FlexWeek {
    fn default() -> FlexWeek {
        let mut w = FlexWeek { days: [Default::default(); 7] };
        let mut wd = Weekday::Mon;
        for day in &mut (w.days) {
            day.set_weekday(wd);
            wd = wd.succ();
        }
        w
    }
}

impl Index<u32> for FlexWeek {
    type Output = FlexDay;
    fn index(&self, idx: u32) -> &FlexDay {
        &self.days[idx as usize]
    }
}

impl IndexMut<u32> for FlexWeek {
    fn index_mut<'a>(&'a mut self, idx: u32) -> &'a mut FlexDay {
        &mut self.days[idx as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_minutes_test() {
        let mut w = FlexWeek { days: [Default::default(); 7] };
        assert_eq!(w.total_minutes(), (8 * 60 - 30) * 7);

        w = Default::default();
        assert_eq!(w.total_minutes(), (8 * 60 - 30) * 5);
    }
}
