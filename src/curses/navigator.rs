use timedata::{FlexMonth, FlexDay, DaysOff, find_last_sunday_for, find_first_monday_of_grid, next_month, prev_month};
use chrono::{Datelike, NaiveDate};
use settings::Settings;
use super::Curses;

pub struct Navigator<'a> {
    pub current_month: FlexMonth,
    pub current_day: NaiveDate,
    pub days_off: DaysOff,
    curses: &'a Curses<'a>,
    settings: &'a Settings,
}

impl<'a> Navigator<'a> {
    pub fn new(cur_day: NaiveDate, curses: &'a Curses, settings: &'a Settings) -> Navigator<'a> {
        Navigator {
            days_off: DaysOff::load(cur_day.year(), &settings),
            current_month: FlexMonth::load(cur_day.year(), cur_day.month(), &settings),
            current_day: cur_day,
            curses: curses,
            settings: &settings
        }
    }

    pub fn get_current_day(&self) -> &FlexDay {
        println!("get_current_day: {}", self.current_day);
        let day_and_week = self.current_month.get_week_with_day(self.current_day.day());
        match day_and_week {
            Some((d, _)) => { d }
            None => { panic!("No selected day, impossible") }
        }
    }

    pub fn init(&mut self) {
        self.curses.print_week_header(self.current_day.month());
        self.curses.print_status(&self.settings, &self.current_month, &self.days_off);
        self.current_day = self.select_day(self.current_day);
    }

    pub fn select_day(&self, date: NaiveDate) -> NaiveDate {
        println!("select_day : {}", date);
        let day_and_week = self.current_month.get_week_with_day(date.day());
        match day_and_week {
            Some((_, w)) => {
                self.curses.print_week(&w, &date);
                date
            }
            None => {
                self.select_day(date.pred())
            }
        }
    }

    pub fn select_prev_day(&mut self) {
        let old = self.current_day;
        self.current_day = self.current_day.pred();
        if old == find_first_monday_of_grid(self.current_month.year, self.current_month.month) {
            self.change_month(false)
        } else {
            self.select_day(self.current_day);
        }
    }

    pub fn select_next_day(&mut self) {
        let old = self.current_day;
        self.current_day = self.current_day.succ();
        if old == find_last_sunday_for(self.current_month.year, self.current_month.month) {
            self.change_month(true)
        } else {
            self.select_day(self.current_day);
        }
    }

    pub fn change_month(&mut self, next: bool) {
        let (y, m) = if next {
            next_month(self.current_month.year, self.current_month.month)
        } else {
            prev_month(self.current_month.year, self.current_month.month)
        };
        self.current_month = FlexMonth::load(y, m, &self.settings);
        self.curses.print_week_header(m);
        self.select_day(self.current_day);
    }
}