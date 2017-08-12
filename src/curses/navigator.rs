use timedata::*;
use chrono::{Datelike, NaiveDate, Weekday, NaiveTime, Duration, Local, Timelike};
use settings::{Settings};
use super::Curses;
use pancurses::{Input, COLOR_PAIR, Window};
use std::ops::{Add, Sub};

pub enum TimeField {
    Hour,
    Minute,
}

pub struct Navigator<'a> {
    current_month: FlexMonth,
    current_day: NaiveDate,
    days_off: DaysOff,
    curses: Curses<'a>,
    settings: Settings,
}

impl<'a> Navigator<'a> {
    pub fn new(cur_day: NaiveDate, screen: &'a Window) -> Navigator<'a> {
        let settings = Settings::load();
        match settings {
            Some(settings) => {
                Navigator {
                    days_off: DaysOff::load(cur_day.year(), &settings),
                    current_month: FlexMonth::load(cur_day.year(), cur_day.month(), &settings),
                    current_day: cur_day,
                    curses: Curses::new(&screen),
                    settings: settings,
                }
            }
            None => {
                // No settings, load defaults and open settings editor
                let settings = Settings::default();
                let mut n = Navigator {
                    days_off: DaysOff::load(cur_day.year(), &settings),
                    current_month: FlexMonth::load(cur_day.year(), cur_day.month(), &settings),
                    current_day: cur_day,
                    curses: Curses::new(&screen),
                    settings: settings,
                };
                n.edit_settings();
                n.current_month = FlexMonth::load(cur_day.year(), cur_day.month(), &n.settings);
                n
            }
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
        self.curses.main_win.clear();
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

    pub fn select_prev_week(&mut self) {
        self.current_day = self.current_day.sub(Duration::days(7));
        if self.current_day <
            find_first_monday_of_grid(self.current_month.year, self.current_month.month) {
            self.change_month(false)
        } else {
            self.select_day(self.current_day);
        }
    }

    pub fn select_next_week(&mut self) {
        self.current_day = self.current_day.add(Duration::days(7));
        if self.current_day >
            find_last_sunday_for(self.current_month.year, self.current_month.month) {
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
                    DayStatus::Sick => self.days_off.sick_days_taken -= 1.0,
                    DayStatus::Weekend | DayStatus::Worked => {},
                },
                DayStatus::Holiday => match new_status {
                    DayStatus::Worked | DayStatus::Weekend => self.days_off.holidays_left += 1.0,
                    DayStatus::Half => self.days_off.holidays_left += 0.5,
                    DayStatus::Sick => {
                        self.days_off.holidays_left += 1.0;
                        self.days_off.sick_days_taken -= 1.0;
                    },
                    DayStatus::Holiday => {}
                },
                DayStatus::Half => match new_status {
                    DayStatus::Worked | DayStatus::Weekend => self.days_off.holidays_left += 0.5,
                    DayStatus::Holiday => self.days_off.holidays_left -= 0.5,
                    DayStatus::Sick => {
                        self.days_off.holidays_left += 0.5;
                        self.days_off.sick_days_taken -= 1.0;
                    },
                    DayStatus::Half => {},
                },
                DayStatus::Sick => {
                    self.days_off.sick_days_taken += 1.0;
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


    fn manage_digit_input_for_time(&self, time: NaiveTime, field: TimeField,
                                   c: char, digit_idx: i32) -> NaiveTime {
        let digit = c.to_digit(10);
        if digit_idx == 0 {
            match digit {
                Some(digit) => match field {
                    TimeField::Hour => time.with_hour(digit)
                        .expect(&format!("something wrong while with_hour with {}", digit)),
                    TimeField::Minute => time.with_minute(digit)
                        .expect(&format!("something wrong while with_minute with {}", digit))
                },
                None => match field {
                    TimeField::Hour => time.with_hour(time.hour() / 10)
                        .expect(&format!("something wrong while with_hour with {}",
                                         time.hour() / 10)),
                    TimeField::Minute => time.with_minute(time.minute() / 10)
                        .expect(&format!("something wrong while with_minute with {}",
                                         time.minute() / 10))
                }
            }
        } else {
            match digit {
                Some(digit) => match field {
                    TimeField::Hour => self.edit_2nd_digit_hour(time, digit),
                    TimeField::Minute => self.edit_2nd_digit_minute(time, digit),
                },
                None => match field {
                    TimeField::Hour => time.with_hour(0).unwrap(),
                    TimeField::Minute => time.with_minute(0).unwrap(),
                }
            }
        }
    }

    fn manage_digit_input_for_duration(&self, duration: i64, field: TimeField,
                                       c: char, digit_idx: i32) -> i64 {
        let digit = c.to_digit(10);
        let nb_hours = duration / 60;
        let nb_hours_in_min = nb_hours * 60;
        let min_left = duration - nb_hours_in_min;
        match digit {
            Some(digit) => {
                let digit64 = digit as i64;
                if digit_idx == 0 {
                    match field {
                        TimeField::Hour => digit64 * 60 + min_left,
                        TimeField::Minute => nb_hours_in_min + digit64
                    }
                } else {
                    match field {
                        TimeField::Hour => {
                            if nb_hours <= 2 && digit <= 3 {
                                nb_hours_in_min * 10 + digit64 * 60 + min_left
                            } else {
                                duration
                            }
                        },
                        TimeField::Minute => {
                            if min_left <= 5 {
                                nb_hours_in_min + min_left * 10 + digit64
                            } else {
                                duration
                            }
                        }
                    }
                }
            },
            None => match field {
                TimeField::Hour => {
                    (nb_hours / 10) * 60 + min_left
                },
                TimeField::Minute => nb_hours_in_min + min_left / 10
            }
        }
    }

    fn manage_digit_input_for_number(&self, nb: f32, c: char, digit_idx: i32) -> f32 {
        let digit = match c.to_digit(10) {
            Some(d) => d,
            None => return nb
        };
        if digit_idx == 0 {
            digit as f32
        } else {
            nb * 10.0 + digit as f32
        }
    }

    fn manage_digit_input(&self, cur_field: usize, c: char, digit_idx: i32, mut d: &mut FlexDay) {
        match cur_field {
            0 => { unreachable!() },
            1 => {
                d.start =
                    self.manage_digit_input_for_time(d.start, TimeField::Hour, c, digit_idx);
            },
            2 => {
                d.start =
                    self.manage_digit_input_for_time(d.start, TimeField::Minute, c, digit_idx);
            }
            3 => {
                d.end = self.manage_digit_input_for_time(d.end, TimeField::Hour, c, digit_idx);
            },
            4 => {
                d.end =
                    self.manage_digit_input_for_time(d.end, TimeField::Minute, c, digit_idx);
            },
            5 => {
                d.pause =
                    self.manage_digit_input_for_duration(d.pause, TimeField::Hour, c, digit_idx);
            },
            6 => {
                d.pause =
                    self.manage_digit_input_for_duration(d.pause, TimeField::Minute, c, digit_idx);
            },
            _ => { unreachable!() },
        }
    }

    fn cur_y_in_week(&self, d: &FlexDay) -> i32 {
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

    pub fn edit_day(&mut self) {
        let mut d = self.get_current_day().clone();
        let now = Local::now().naive_utc();
        let cur_y = self.cur_y_in_week(&d);
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
        self.curses.week_win.refresh();

        let mut done = false;
        let mut digit_idx = 0;
        while !done {
            let old_status = d.status;
            match self.curses.getch() {
                Some(c) => {
                    match c {
                        Input::Character('\x1B') | Input::Character('\n') => done = true,
                        Input::KeyRight => {
                            digit_idx = 0;
                            if cur_field < self.curses.fields.len() - 1 {
                                cur_field += 1;
                                self.curses.highlight_current_field(cur_field, &d, cur_y);
                            }
                        },
                        Input::KeyLeft => {
                            digit_idx = 0;
                            if cur_field > 0 {
                                cur_field -= 1;
                                self.curses.highlight_current_field(cur_field, &d, cur_y);
                            }
                        },
                        Input::KeyUp | Input::KeyDown => {
                            digit_idx = 0;
                            self.manage_key_up_down(cur_field, c == Input::KeyUp, &mut d);
                            self.curses.highlight_current_field(cur_field, &d, cur_y);
                        },
                        Input::Character(c) if (c >= '0' && c <= '9') || c == '\u{8}' => {
                            if cur_field > 0 {
                                self.manage_digit_input(cur_field, c, digit_idx, &mut d);
                                digit_idx = (digit_idx + 1) % 2;
                                self.curses.highlight_current_field(cur_field, &d, cur_y);
                            }
                        },
                        _ => { println!("unknown: {:?}", c) }
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
                    'h' => if d.status == DayStatus::Holiday {
                        DayStatus::Worked
                    } else {
                        DayStatus::Holiday
                    },
                    's' => if d.status == DayStatus::Sick {
                        DayStatus::Worked
                    } else {
                        DayStatus::Sick
                    },
                    _ => d.status
                };
                self.update_display_post_edit(old_status, d);
                self.curses.week_win.mv(self.cur_y_in_week(&d), 0);
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

    pub fn edit_settings(&mut self) {
        self.curses.open_settings(&self.settings, &self.days_off);
        let mut cur_idx = 0;
        let mut cur_field = 0;
        let mut done = false;
        self.select_option(cur_idx, cur_field);
        let mut digit_idx = 0;
        while !done {
            match self.curses.getch() {
                Some(c) => {
                    match c {
                        Input::Character('\x1B') => done = true,
                        Input::KeyUp => {
                            digit_idx = 0;
                            if cur_idx <= 0 {
                                if cur_field <= 5 {
                                    cur_idx = 4;
                                } else {
                                    cur_idx = 2;
                                }
                            } else {
                                cur_idx -= 1;
                            }
                            self.select_option(cur_idx, cur_field);
                        },
                        Input::KeyDown => {
                            digit_idx = 0;
                            if cur_field <= 5 {
                                cur_idx = (cur_idx + 1) % 5;
                            } else {
                                cur_idx = (cur_idx + 1) % 3;
                            }
                            self.select_option(cur_idx, cur_field);
                        },
                        Input::KeyLeft => {
                            digit_idx = 0;
                            if cur_field > 0 {
                                cur_field -= 1;
                            } else {
                                cur_field = 6;
                                if cur_idx > 2 { cur_idx = 2; }
                            }
                            self.select_option(cur_idx, cur_field);
                        },
                        Input::KeyRight => {
                            digit_idx = 0;
                            cur_field = (cur_field + 1) % 7;
                            if cur_field > 5 {
                                if cur_idx > 2 { cur_idx = 2; }
                            }
                            self.select_option(cur_idx, cur_field);
                        },
                        Input::Character(c) if c >= '0' && c <= '9' => {
                            self.manage_option_edition(cur_idx, cur_field, c, digit_idx);
                            self.select_option(cur_idx, cur_field);
                            digit_idx = (digit_idx + 1) % 2;
                        },
                        _ => {}
                    }
                },
                None => {}
            }
        }
        self.settings.save();
        self.days_off.save();
        self.curses.close_setting();
        self.init();
    }

    fn select_option(&mut self, cur_idx: i32, cur_field: i32) {
        self.curses.highlight_option(cur_idx, cur_field, &self.settings, &self.days_off);
    }

    fn manage_option_edition(&mut self, cur_idx: i32, cur_field: i32, c: char, digit_idx: i32) {
        match cur_field {
            sched_field if sched_field <= 5 => {
                let mut d = self.settings.week_sched.sched[cur_idx as usize];
                match sched_field {
                    0 => d.start =
                        self.manage_digit_input_for_time(d.start, TimeField::Hour, c, digit_idx),
                    1 => d.start =
                        self.manage_digit_input_for_time(d.start, TimeField::Minute, c, digit_idx),
                    2 => d.end =
                        self.manage_digit_input_for_time(d.end, TimeField::Hour, c, digit_idx),
                    3 => d.end =
                        self.manage_digit_input_for_time(d.end, TimeField::Minute, c, digit_idx),
                    4 => d.pause =
                        self.manage_digit_input_for_duration(d.pause, TimeField::Hour, c,
                                                             digit_idx),
                    5 => d.pause =
                        self.manage_digit_input_for_duration(d.pause, TimeField::Minute, c,
                                                             digit_idx),
                    _ => unreachable!()
                };
                self.settings.week_sched.sched[cur_idx as usize] = d;
            },
            6 => {
                match cur_idx {
                    0 => {
                        self.settings.holidays_per_year =
                            self.manage_digit_input_for_number(self.settings.holidays_per_year,
                                                               c, digit_idx);
                    },
                    1 => {
                        self.days_off.holidays_left =
                            self.manage_digit_input_for_number(self.days_off.holidays_left,
                                                               c, digit_idx);
                    },
                    2 => {
                        self.days_off.sick_days_taken =
                            self.manage_digit_input_for_number(self.days_off.sick_days_taken,
                                                               c, digit_idx);
                    },
                    _ => unreachable!()
                }
            },
            _ => unreachable!()
        }
    }
}
