use timedata::*;
use chrono::{Datelike, NaiveDate, Weekday, NaiveTime, Duration, Local};
use settings::Settings;
use super::Curses;
use pancurses::Input;
use std::ops::{Add, Sub};

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
        let day_and_week = self.current_month.get_week_with_day(self.current_day.day());
        match day_and_week {
            Some((d, _)) => { d }
            None => { unreachable!("No selected day, impossible") }
        }
    }

    pub fn init(&mut self) {
        self.curses.print_week_header(self.current_day.month());
        self.curses.print_status(&self.settings, &self.current_month, &self.days_off);
        self.current_day = self.select_day(self.current_day);
    }

    pub fn select_day(&self, date: NaiveDate) -> NaiveDate {
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

    fn edit_status(d: &mut FlexDay, up: bool) {
        let wd = d.weekday().expect("should have weekday");
        d.status = if up {
            match d.status {
                DayStatus::Worked | DayStatus::Holiday => DayStatus::Worked,
                DayStatus::Half => DayStatus::Holiday,
                DayStatus::Sick => DayStatus::Half,
                DayStatus::Weekend => DayStatus::Worked,
            }
        } else {
            match d.status {
                DayStatus::Worked => match wd {
                    Weekday::Sat | Weekday::Sun => DayStatus::Weekend,
                    _ => DayStatus::Holiday,
                },
                DayStatus::Holiday => DayStatus::Half,
                DayStatus::Half => DayStatus::Sick,
                DayStatus::Sick => DayStatus::Sick,
                DayStatus::Weekend => DayStatus::Weekend,
            }
        }
    }

    fn edit_hour(time: NaiveTime, up: bool) -> NaiveTime {
        let offset = Duration::hours(1);
        if up { time.add(offset) } else { time.sub(offset) }
    }

    fn edit_minute(time: NaiveTime, up: bool) -> NaiveTime {
        let offset = Duration::minutes(1);
        if up { time.add(offset) } else { time.sub(offset) }
    }

    fn manage_key_up_down(&self, cur_field: usize, up: bool, mut d: &mut FlexDay) {
        match cur_field {
            0 => { Navigator::edit_status(&mut d, up); },
            1 => { d.start = Navigator::edit_hour(d.start, up); },
            2 => { d.start = Navigator::edit_minute(d.start, up); }
            3 => { d.end = Navigator::edit_hour(d.end, up); },
            4 => { d.end = Navigator::edit_minute(d.end, up); },
            5 => {},
            6 => {},
            _ => { unreachable!() },
        }
    }

    pub fn manage_edit(&mut self) {
        let mut d = self.get_current_day().clone();
        let now = Local::now().naive_utc();
        let cur_y =
            match d.weekday().expect("weekday not set, impossible") {
                Weekday::Mon => 2,
                Weekday::Tue => 3,
                Weekday::Wed => 4,
                Weekday::Thu => 5,
                Weekday::Fri => 6,
                Weekday::Sat => 7,
                Weekday::Sun => 8,
            };
        let mut cur_field: usize =
            match d.status {
                DayStatus::Weekend | DayStatus::Sick | DayStatus::Holiday => 0,
                _ => if now.time() < NaiveTime::from_hms(12, 00, 00) {
                    2 // set to start min field
                } else {
                    4 // set to end min field
                },
            };

        self.curses.highlight_current_field(cur_field, &d, cur_y);

        let mut done = false;
        while !done {
            match self.curses.getch() {
                Some(c) => {
                    match c {
                        Input::KeyRight => {
                            if cur_field < self.curses.fields.len() - 1 {
                                cur_field += 1;
                                self.curses.highlight_current_field(cur_field, &d, cur_y);
                            }
                        },
                        Input::KeyLeft => {
                            if cur_field > 0 {
                                cur_field -= 1;
                                self.curses.highlight_current_field(cur_field, &d, cur_y);
                            }
                        },
                        Input::KeyUp => {
                            self.manage_key_up_down(cur_field, true, &mut d);
                            self.curses.highlight_current_field(cur_field, &d, cur_y);
                        },
                        Input::KeyDown => {
                            self.manage_key_up_down(cur_field, false, &mut d);
                            self.curses.highlight_current_field(cur_field, &d, cur_y);
                        },
                        Input::Character('\x1B') => done = true,
                        Input::Character(c) => {
                            println!("{:?}", c);
                        }
                        _ => { println!("unknown") }
                    }
                }
                None => {}
            }
            let idx = match d.weekday().expect("weekday not set, impossible") {
                Weekday::Mon => 0,
                Weekday::Tue => 1,
                Weekday::Wed => 2,
                Weekday::Thu => 3,
                Weekday::Fri => 4,
                Weekday::Sat => 5,
                Weekday::Sun => 6,
            };
            // todo update data and save
            self.current_month.update_day(d);
            self.current_month.update_balance();
            self.current_month.save();
            self.curses.print_status(&self.settings, &self.current_month, &self.days_off);
        }
        // remove any reverse attr
        self.select_day(self.current_day);
    }
}