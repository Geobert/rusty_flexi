use std::iter::Iterator;
use chrono::{ NaiveDate };
pub struct NaiveDateIter {
    start: NaiveDate,
    end: NaiveDate
}

impl NaiveDateIter {
    pub fn new(start: NaiveDate, end: NaiveDate) -> NaiveDateIter {
        NaiveDateIter { start: start, end: end }
    }
}

impl Iterator for NaiveDateIter {
    type Item = NaiveDate;
    fn next(&mut self) -> Option<NaiveDate> {
        
        if self.start.gt(&self.end) {
            None 
        } else {
            let res = self.start;
            self.start = self.start.succ();
            Some(res)
        }
    }
}

#[test]
fn naivedate_iter_test() {
    let first_monday = NaiveDate::from_ymd(2017, 04, 01);
    let last_sunday = NaiveDate::from_ymd(2017, 04, 30);
    let mut itr = NaiveDateIter::new(first_monday, last_sunday);
    let mut count = 0;
    while let Some(_) = itr.next() {
        count += 1;
    }
    assert_eq!(count, 30);
}