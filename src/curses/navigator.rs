use timedata::*;
use chrono::{Datelike, NaiveDate, Weekday, NaiveTime, Duration, Local};
use settings::{Settings};
use super::Curses;
use pancurses::{Input, COLOR_PAIR, Window};
use std::ops::{Add, Sub};
use super::editor;
use super::editor::TimeField;

pub struct Navigator<'a> {
    current_month: FlexMonth,
    current_day: NaiveDate,
    days_off: DaysOff,
    curses: Curses<'a>,
    settings: Settings,
}

pub enum HourField {
    Begin,
    End
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
        let day_and_week = self.current_month.get_week_with_day(self.current_day);
        match day_and_week {
            Some((d, _, _)) => { d }
            None => { unreachable!("No selected day, impossible") }
        }
    }

    pub fn init(&mut self) {
        self.curses.main_win.clear();
        self.curses.print_status(&self.settings, &self.current_month, &self.days_off);
        let date = self.current_day;
        self.current_day = self.select_day(date);
    }

    fn first_day_of_month_at_current_weekday(&self) -> NaiveDate {
        self.current_month.weeks[0]
            [self.current_day.weekday().num_days_from_monday()].date
            .expect("change_month: should have date")
    }

    fn last_day_of_month_at_current_weekday(&self) -> NaiveDate {
        self.current_month.weeks[self.current_month.weeks.len() - 1]
            [self.current_day.weekday().num_days_from_monday()].date
            .expect("change_month: should have date")
    }

    fn select_day_in_month(&self, date: NaiveDate, month: &FlexMonth) -> Option<NaiveDate> {
        let day_and_week = month.get_week_with_day(date);
        match day_and_week {
            Some((_, w, week_nb)) => {
                self.curses.print_week_header(&month, week_nb);
                self.curses.print_week(&w, &date);
                self.curses.print_week_total(&w, w.total_minutes() < self.settings.week_goal);
                Some(date)
            }
            None => {
                None
            }
        }
    }

    pub fn select_day(&mut self, date: NaiveDate) -> NaiveDate {
        let month = self.current_month.clone();
        match self.select_day_in_month(date, &month) {
            Some(date) => {
                self.current_day = date;
                date
            },
            None => {
                self.current_month = FlexMonth::load(date.year(), date.month(), &self.settings);
                self.select_day(date)
            }
        }
    }

    pub fn select_prev_day(&mut self) {
        let old = self.current_day;
        self.current_day = self.current_day.pred();
        if old == find_first_monday_of_grid(self.current_month.year, self.current_month.month) {
            self.change_month(false)
        } else {
            let date = self.current_day;
            self.select_day(date);
        }
    }

    pub fn select_next_day(&mut self) {
        let old = self.current_day;
        self.current_day = self.current_day.succ();
        if old == find_last_sunday_for(self.current_month.year, self.current_month.month) {
            self.change_month(true)
        } else {
            let date = self.current_day;
            self.select_day(date);
        }
    }

    pub fn select_prev_week(&mut self) {
        self.current_day = self.current_day.sub(Duration::days(7));
        if self.current_day <
            find_first_monday_of_grid(self.current_month.year, self.current_month.month) {
            self.change_month(false)
        } else {
            let date = self.current_day;
            self.select_day(date);
        }
    }

    pub fn select_next_week(&mut self) {
        self.current_day = self.current_day.add(Duration::days(7));
        if self.current_day >
            find_last_sunday_for(self.current_month.year, self.current_month.month) {
            self.change_month(true)
        } else {
            let date = self.current_day;
            self.select_day(date);
        }
    }

    pub fn change_month(&mut self, next: bool) {
        let (y, m) = if next {
            next_month(self.current_month.year, self.current_month.month)
        } else {
            prev_month(self.current_month.year, self.current_month.month)
        };
        self.current_month = FlexMonth::load(y, m, &self.settings);
        let date = if next {
            self.first_day_of_month_at_current_weekday()
        } else {
            self.last_day_of_month_at_current_weekday()
        };
        self.current_day = self.select_day(date);
        self.curses.print_status(&self.settings, &self.current_month, &self.days_off);
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
                            editor::process_key_up_down(cur_field, c == Input::KeyUp, &mut d);
                            self.curses.highlight_current_field(cur_field, &d, cur_y);
                        },
                        Input::Character(c) if (c >= '0' && c <= '9') || c == '\u{8}' => {
                            if cur_field > 0 {
                                editor::process_digit_input(cur_field, c, digit_idx, &mut d);
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
        let date = self.current_day;
        self.select_day(date);
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
                self.update_display_post_direct_edit(old_status, d);
            }
        }
    }

    fn update_display_post_direct_edit(&mut self, old_status: DayStatus, d: FlexDay) {
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

    pub fn change_time(&mut self, time: NaiveTime, field: HourField) {
        let mut d = self.get_current_day().clone();
        match d.status {
            DayStatus::Worked | DayStatus::Half => {
                match field {
                    HourField::Begin => d.start = time,
                    HourField::End => d.end = time
                }
                self.update_display_post_direct_edit(d.status, d);
            },
            _ => {}
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
                Some(Input::Character('\x1B')) => done = true,
                Some(c) => {
                    match c {
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
                        },
                        Input::KeyDown => {
                            digit_idx = 0;
                            if cur_field <= 5 {
                                cur_idx = (cur_idx + 1) % 6;
                            } else {
                                cur_idx = (cur_idx + 1) % 3;
                            }
                            self.select_option(cur_idx, cur_field)
                        },
                        Input::KeyLeft => {
                            digit_idx = 0;
                            if cur_field > 0 {
                                cur_field -= 1;
                            } else {
                                cur_field = 6;
                                if cur_idx > 2 { cur_idx = 2; }
                            }
                            self.select_option(cur_idx, cur_field)
                        },
                        Input::KeyRight => {
                            digit_idx = 0;
                            cur_field = (cur_field + 1) % 7;
                            if cur_field > 5 {
                                if cur_idx > 2 { cur_idx = 2; }
                            }
                            self.select_option(cur_idx, cur_field)
                        },
                        Input::Character(c) if c >= '0' && c <= '9' => {
                            self.manage_option_edition(cur_idx, cur_field, c, digit_idx);
                            digit_idx = (digit_idx + 1) % 2;
                            self.select_option(cur_idx, cur_field)
                        },
                        _ => {}
                    };
                    cur_field =
                        if cur_idx == 5 {
                            if cur_field < 4 { 4 } else if cur_field > 5 { 5 } else { cur_field }
                        } else { cur_field };
                    self.select_option(cur_idx, cur_field)
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
        self.curses.highlight_option(cur_idx, cur_field, &self.settings, &self.days_off)
    }

    fn manage_option_edition(&mut self, cur_idx: i32, cur_field: i32, c: char, digit_idx: i32) {
        if cur_idx < 5 {
            match cur_field {
                sched_field if sched_field <= 5 => {
                    let mut d = self.settings.week_sched.sched[cur_idx as usize];
                    match sched_field {
                        0 => d.start =
                            editor::process_digit_input_for_time(d.start, TimeField::Hour, c, digit_idx),
                        1 => d.start =
                            editor::process_digit_input_for_time(d.start, TimeField::Minute, c, digit_idx),
                        2 => d.end =
                            editor::process_digit_input_for_time(d.end, TimeField::Hour, c, digit_idx),
                        3 => d.end =
                            editor::process_digit_input_for_time(d.end, TimeField::Minute, c, digit_idx),
                        4 => d.pause =
                            editor::process_digit_input_for_duration(d.pause, TimeField::Hour, c,
                                                                     digit_idx),
                        5 => d.pause =
                            editor::process_digit_input_for_duration(d.pause, TimeField::Minute, c,
                                                                     digit_idx),
                        _ => unreachable!()
                    };
                    self.settings.week_sched.sched[cur_idx as usize] = d;
                },
                6 => {
                    match cur_idx {
                        0 => {
                            self.settings.holidays_per_year =
                                editor::process_digit_input_for_number(self.settings.holidays_per_year,
                                                                       c, digit_idx);
                        },
                        1 => {
                            self.days_off.holidays_left =
                                editor::process_digit_input_for_number(self.days_off.holidays_left,
                                                                       c, digit_idx);
                        },
                        2 => {
                            self.days_off.sick_days_taken =
                                editor::process_digit_input_for_number(self.days_off.sick_days_taken,
                                                                       c, digit_idx);
                        },
                        _ => unreachable!()
                    }
                },
                _ => unreachable!()
            }
        } else {
            match cur_field {
                4 => {
                    self.settings.week_goal =
                        editor::process_digit_input_for_duration(self.settings.week_goal,
                                                                 TimeField::Hour,
                                                                 c, digit_idx)
                },
                5 => {
                    self.settings.week_goal =
                        editor::process_digit_input_for_duration(self.settings.week_goal,
                                                                 TimeField::Minute,
                                                                 c, digit_idx)
                },
                _ => { unreachable!() }
            }
            self.settings.holiday_duration = self.settings.week_goal / 5;
        }
    }
}
