use timedata::*;
use chrono::{Datelike, NaiveDate, Weekday, NaiveTime, Duration, Local, Timelike};
use settings::Settings;
use super::Curses;
use pancurses::{Input, COLOR_PAIR};
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
        self.curses.print_week_header(self.current_day.month(), self.current_day.year());
        self.curses.print_status(&self.settings, &self.current_month, &self.days_off);

        self.current_day = self.select_day(self.current_day);
    }

    pub fn select_day(&self, date: NaiveDate) -> NaiveDate {
        let day_and_week = self.current_month.get_week_with_day(date.day());
        match day_and_week {
            Some((_, w)) => {
                self.curses.print_week(&w, &date);
                self.curses.print_week_total(&w, w.total_minutes() < self.settings.week_goal);
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
        self.curses.print_week_header(m, y);
        self.select_day(self.current_day);
        self.curses.print_status(&self.settings, &self.current_month, &self.days_off);
    }

    fn scroll_status(&self, d: &mut FlexDay, up: bool) {
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

    fn add_to_hour(&self, time: NaiveTime, up: bool, nb: i64) -> NaiveTime {
        let duration = Duration::hours(nb);
        if up { time.add(duration) } else { time.sub(duration) }
    }

    fn add_to_minute(&self, time: NaiveTime, up: bool, nb: i64) -> NaiveTime {
        let duration = Duration::minutes(nb);
        if up { time.add(duration) } else { time.sub(duration) }
    }

    fn manage_key_up_down(&self, cur_field: usize, up: bool, mut d: &mut FlexDay) {
        match cur_field {
            0 => { self.scroll_status(&mut d, up); },
            1 => { d.start = self.add_to_hour(d.start, up, 1); },
            2 => { d.start = self.add_to_minute(d.start, up, 1); }
            3 => { d.end = self.add_to_hour(d.end, up, 1); },
            4 => { d.end = self.add_to_minute(d.end, up, 1); },
            5 => { d.pause += if up { 60 } else { -60 } },
            6 => { d.pause += if up { 1 } else { -1 } },
            _ => { unreachable!() },
        }
    }

    fn update_days_off(&mut self, old_status: DayStatus, new_status: DayStatus) {
        if old_status != new_status {
            match old_status {
                DayStatus::Worked => match new_status {
                    DayStatus::Holiday => self.days_off.holidays_left -= 1.0,
                    DayStatus::Half => self.days_off.holidays_left -= 0.5,
                    DayStatus::Sick => self.days_off.sick_days_left -= 1.0,
                    DayStatus::Weekend | DayStatus::Worked => {},
                },
                DayStatus::Holiday => match new_status {
                    DayStatus::Worked | DayStatus::Weekend => self.days_off.holidays_left += 1.0,
                    DayStatus::Half => self.days_off.holidays_left += 0.5,
                    DayStatus::Sick => {
                        self.days_off.holidays_left += 1.0;
                        self.days_off.sick_days_left -= 1.0;
                    },
                    DayStatus::Holiday => {}
                },
                DayStatus::Half => match new_status {
                    DayStatus::Worked | DayStatus::Weekend => self.days_off.holidays_left += 0.5,
                    DayStatus::Holiday => self.days_off.holidays_left -= 0.5,
                    DayStatus::Sick => {
                        self.days_off.holidays_left += 0.5;
                        self.days_off.sick_days_left -= 1.0;
                    },
                    DayStatus::Half => {},
                },
                DayStatus::Sick => {
                    self.days_off.sick_days_left += 1.0;
                    match new_status {
                        DayStatus::Half => self.days_off.holidays_left -= 0.5,
                        DayStatus::Holiday => self.days_off.holidays_left -= 1.0,
                        DayStatus::Sick => {},
                        DayStatus::Worked | DayStatus::Weekend => {}
                    }
                },
                DayStatus::Weekend => {},
            }
        }
    }

    fn edit_2nd_digit_hour(&self, time: NaiveTime, digit: u32) -> NaiveTime {
        match time.hour() {
            1 => {
                time.with_hour(time.hour() * 10 + digit)
                    .expect(&format!("something wrong while with_hour with {}",
                                     time.hour() * 10 + digit))
            },
            2 if digit <= 3 => {
                time.with_hour(time.hour() * 10 + digit)
                    .expect(&format!("something wrong while with_hour with {}",
                                     time.hour() * 10 + digit))
            },
            _ => { time }
        }
    }

    fn edit_2nd_digit_minute(&self, time: NaiveTime, digit: u32) -> NaiveTime {
        if time.minute() <= 5 {
            time.with_minute(time.minute() * 10 + digit)
                .expect(&format!("something wrong while with_minute with {}",
                                 time.minute() * 10 + digit))
        } else {
            time
        }
    }

    fn manage_digit_input(&self, cur_field: usize, c: char, digit_idx: i32, mut d: &mut FlexDay) {
        let digit = c.to_digit(10).unwrap();
        let digit64 = digit as i64;
        if digit_idx == 0 {
            match cur_field {
                0 => { unreachable!() },
                1 => {
                    d.start = d.start.with_hour(digit)
                        .expect(&format!("something wrong while with_hour with {}", digit));
                },
                2 => {
                    d.start = d.start.with_minute(digit)
                        .expect(&format!("something wrong while with_minute with {}", digit));
                }
                3 => {
                    d.end = d.end.with_hour(digit)
                        .expect(&format!("something wrong while with_hour with {}", digit));
                },
                4 => {
                    d.end = d.end.with_minute(digit)
                        .expect(&format!("something wrong while with_minute with {}", digit));
                },
                5 => { d.pause = digit64 * 60; },
                6 => { d.pause = digit64; },
                _ => { unreachable!() },
            }
        } else {
            match cur_field {
                0 => { unreachable!() },
                1 => { d.start = self.edit_2nd_digit_hour(d.start, digit); },
                2 => { d.start = self.edit_2nd_digit_minute(d.start, digit); },
                3 => { d.end = self.edit_2nd_digit_hour(d.end, digit); },
                4 => { d.end = self.edit_2nd_digit_minute(d.end, digit); },
                5 => {
                    if d.pause <= 2 * 60 && digit <= 3 {
                        d.pause = d.pause * 10 + digit64 * 60;
                    }
                },
                6 => {
                    if d.pause <= 5 {
                        d.pause = d.pause * 10 + digit64;
                    }
                },
                _ => { unreachable!() },
            }
        }
    }

    fn cur_y(&self, d: &FlexDay) -> i32 {
        match d.weekday().expect("weekday not set, impossible") {
            Weekday::Mon => 2,
            Weekday::Tue => 3,
            Weekday::Wed => 4,
            Weekday::Thu => 5,
            Weekday::Fri => 6,
            Weekday::Sat => 7,
            Weekday::Sun => 8,
        }
    }

    pub fn manage_edit(&mut self) {
        let mut d = self.get_current_day().clone();
        let now = Local::now().naive_utc();
        let cur_y = self.cur_y(&d);
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
        let mut digit_idx = 0;
        while !done {
            let old_status = d.status;
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
                        Input::Character('\x1B') | Input::Character('\n') => done = true,

                        Input::Character(c) if c >= '0' || c <= '9' => {
                            if cur_field > 0 {
                                self.manage_digit_input(cur_field, c, digit_idx, &mut d);
                                digit_idx = (digit_idx + 1) % 2;
                                self.curses.highlight_current_field(cur_field, &d, cur_y);
                            }
                        }
                        _ => { println!("unknown") }
                    }
                }
                None => {}
            }
            self.update_display_post_edit(old_status, d);
        }
        // remove any reverse attr
        self.select_day(self.current_day);
    }

    pub fn change_status(&mut self, c: char) {
        let mut d = self.get_current_day().clone();
        let old_status = d.status;
        match d.weekday().expect("must have weekday") {
            Weekday::Sat | Weekday::Sun => {},
            _ => {
                d.status = match c {
                    'h' => if d.status == DayStatus::Holiday { DayStatus::Worked } else { DayStatus::Holiday },
                    's' => if d.status == DayStatus::Sick { DayStatus::Worked } else { DayStatus::Sick },
                    _ => d.status
                };
                self.update_display_post_edit(old_status, d);
                self.curses.week_win.mv(self.cur_y(&d), 0);
                if d.total_minutes() < 0 {
                    self.curses.week_win.attron(COLOR_PAIR(1));
                }
                self.curses.print_selected_day(&d);
                if d.total_minutes() < 0 {
                    self.curses.week_win.attroff(COLOR_PAIR(1));
                }
                self.curses.week_win.refresh();
            }
        }
    }

    fn update_display_post_edit(&mut self, old_status: DayStatus, d: FlexDay) {
        self.update_days_off(old_status, d.status);
        let week = self.current_month.update_day(d).expect("Should find a week");
        self.current_month.update_balance();
        self.current_month.save();
        self.days_off.save();
        self.curses.print_status(&self.settings, &self.current_month, &self.days_off);
        self.curses.print_week_total(&week,
                                     week.total_minutes() < self.settings.week_goal);
    }
}
