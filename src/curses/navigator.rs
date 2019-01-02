use super::editor;
use super::editor::TimeField;
use super::Curses;
use crate::settings::Settings;
use crate::timedata::*;
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveTime, Timelike, Weekday};
use failure::Error;
use pancurses::{Input, Window, COLOR_PAIR};
use std::ops::{Add, Sub};

pub struct Navigator<'a> {
    current_month: FlexMonth,
    current_day: NaiveDate,
    pub days_off: DaysOff,
    curses: Curses<'a>,
}

pub enum HourField {
    Begin,
    End,
}

#[derive(PartialEq)]
pub enum Direction {
    Previous,
    Next,
}

impl<'a> Navigator<'a> {
    pub fn new(cur_day: NaiveDate, screen: &'a Window, settings: &Settings) -> Self {
        let mut nav = Navigator {
            days_off: DaysOff::load(cur_day.year(), &settings),
            current_month: FlexMonth::load(cur_day.year(), cur_day.month(), &settings),
            current_day: cur_day,
            curses: Curses::new(&screen),
        };

        // let mut nav = match settings {
        //     Some(settings) =>
        //     },
        //     None => {
        //         // No settings, load defaults and open settings editor
        //         let settings = Settings::default();
        //         let mut n = Navigator {
        //             days_off: DaysOff::load(cur_day.year(), &settings),
        //             current_month: FlexMonth::load(cur_day.year(), cur_day.month(), &settings),
        //             current_day: cur_day,
        //             curses: Curses::new(&screen),
        //             settings: settings,
        //         };
        //         n.edit_settings()?;
        //         n.current_month = FlexMonth::load(cur_day.year(), cur_day.month(), &n.settings);
        //         n
        //     }
        // };
        nav.days_off.roll_sick_days();
        nav
    }

    pub fn get_current_day(&self) -> &FlexDay {
        let day_and_week = self.current_month.get_week_with_day(self.current_day);
        match day_and_week {
            Some((d, _, _)) => d,
            None => unreachable!("No selected day, impossible"),
        }
    }

    pub fn init(&mut self, settings: &Settings) {
        self.curses.main_win.clear();
        let date = self.current_day;
        self.current_day = self.select_day(date, &settings);
        self.curses
            .print_status(&settings, &self.current_month, &self.days_off);
    }

    fn first_day_of_month_at_current_weekday(&self) -> NaiveDate {
        self.current_month.weeks[0][self.current_day.weekday().num_days_from_monday()]
            .date
            .expect("change_month: should have date")
    }

    fn last_day_of_month_at_current_weekday(&self) -> NaiveDate {
        self.current_month.weeks[self.current_month.weeks.len() - 1]
            [self.current_day.weekday().num_days_from_monday()]
        .date
        .expect("change_month: should have date")
    }

    fn select_day_in_month(
        &self,
        date: NaiveDate,
        month: &FlexMonth,
        settings: &Settings,
    ) -> Option<NaiveDate> {
        let day_and_week = month.get_week_with_day(date);
        match day_and_week {
            Some((_, w, week_nb)) => {
                self.curses.print_week_header(&month, week_nb);
                self.curses.print_week(&w, &date);
                self.curses
                    .print_week_total(&w, w.total_minutes() < settings.week_goal);
                Some(date)
            }
            None => None,
        }
    }

    pub fn select_day(&mut self, date: NaiveDate, settings: &Settings) -> NaiveDate {
        let cur_month = self.current_month.clone();
        match self.select_day_in_month(date, &cur_month, &settings) {
            Some(date) => {
                self.current_day = date;
                date
            }
            None => {
                let (year, month) = if date.month() < cur_month.month {
                    prev_month(cur_month.year, cur_month.month)
                } else {
                    next_month(cur_month.year, cur_month.month)
                };
                self.current_month = FlexMonth::load(year, month, &settings);
                self.select_day(date, &settings)
            }
        }
    }

    pub fn select_prev_day(&mut self, settings: &Settings) {
        let old = self.current_day;
        self.current_day = self.current_day.pred();
        if old == find_first_monday_of_grid(self.current_month.year, self.current_month.month) {
            self.change_month(Direction::Previous, &settings)
        } else {
            let date = self.current_day;
            self.select_day(date, &settings);
        }
    }

    pub fn select_next_day(&mut self, settings: &Settings) {
        let old = self.current_day;
        self.current_day = self.current_day.succ();
        if old == find_last_sunday_for(self.current_month.year, self.current_month.month) {
            self.change_month(Direction::Next, &settings)
        } else {
            let date = self.current_day;
            self.select_day(date, &settings);
        }
    }

    pub fn select_prev_week(&mut self, settings: &Settings) {
        self.current_day = self.current_day.sub(Duration::days(7));
        if self.current_day
            < find_first_monday_of_grid(self.current_month.year, self.current_month.month)
        {
            self.change_month(Direction::Previous, &settings)
        } else {
            let date = self.current_day;
            self.select_day(date, &settings);
        }
    }

    pub fn select_next_week(&mut self, settings: &Settings) {
        self.current_day = self.current_day.add(Duration::days(7));
        if self.current_day
            > find_last_sunday_for(self.current_month.year, self.current_month.month)
        {
            self.change_month(Direction::Next, &settings)
        } else {
            let date = self.current_day;
            self.select_day(date, &settings);
        }
    }

    pub fn change_month(&mut self, direction: Direction, settings: &Settings) {
        let next = direction == Direction::Next;
        let (y, m) = if next {
            next_month(self.current_month.year, self.current_month.month)
        } else {
            prev_month(self.current_month.year, self.current_month.month)
        };
        self.current_month = FlexMonth::load(y, m, &settings);
        let date = if next {
            self.first_day_of_month_at_current_weekday()
        } else {
            self.last_day_of_month_at_current_weekday()
        };
        self.current_day = self.select_day(date, &settings);
        self.curses
            .print_status(&settings, &self.current_month, &self.days_off);
    }

    pub fn edit_day(&mut self, settings: &Settings) -> Result<(), Error> {
        let mut d = self.get_current_day().clone();
        let selected_day = d.date.expect("edit_day: must have date");
        let now = Local::now().naive_utc();
        let today = now.date();
        let now = NaiveTime::from_hms(now.time().hour(), now.time().minute(), 0);
        let cur_y = self.curses.cur_y_in_week(&d);
        let mut cur_field: usize = match d.status {
            DayStatus::Weekend | DayStatus::Sick | DayStatus::Holiday => 0,
            _ => {
                if selected_day > today || now < NaiveTime::from_hms(12, 00, 00) {
                    2
                } else {
                    4 // set to end min field
                }
            }
        };

        self.curses.highlight_current_field(cur_field, &d, cur_y);
        self.curses.week_win.refresh();

        let mut done = false;
        let mut go_to_today = false;
        let mut digit_idx = 0;
        while !done {
            let old_status = d.status;
            match self.curses.getch() {
                Some(c) => match c {
                    Input::Character('\x1B') | Input::Character('\n') => done = true,
                    Input::KeyHome => {
                        done = true;
                        go_to_today = true;
                    }
                    Input::KeyRight => {
                        digit_idx = 0;
                        if cur_field < self.curses.fields.len() - 1 {
                            cur_field += 1;
                            self.curses.highlight_current_field(cur_field, &d, cur_y);
                        }
                    }
                    Input::KeyLeft => {
                        digit_idx = 0;
                        if cur_field > 0 {
                            cur_field -= 1;
                            self.curses.highlight_current_field(cur_field, &d, cur_y);
                        }
                    }
                    Input::KeyUp | Input::KeyDown => {
                        digit_idx = 0;
                        editor::process_key_up_down(cur_field, c == Input::KeyUp, &mut d);
                        self.curses.highlight_current_field(cur_field, &d, cur_y);
                    }
                    Input::Character(c) if (c >= '0' && c <= '9') || c == '\u{8}' => {
                        if cur_field > 0 {
                            editor::process_digit_input(cur_field, c, digit_idx, &mut d);
                            digit_idx = (digit_idx + 1) % 2;
                            self.curses.highlight_current_field(cur_field, &d, cur_y);
                        }
                    }
                    _ => println!("unknown: {:?}", c),
                },
                None => {}
            }
            self.update_display_post_edit(old_status, d, &settings)?;
        }
        // remove any reverse attr
        let cur_day = self.current_day;
        self.select_day(
            if go_to_today {
                Local::today().naive_local()
            } else {
                cur_day
            },
            &settings,
        );
        Ok(())
    }

    pub fn change_status(&mut self, c: char, settings: &Settings) -> Result<(), Error> {
        let mut d = self.get_current_day().clone();
        let old_status = d.status;
        match d.weekday().expect("must have weekday") {
            Weekday::Sat | Weekday::Sun => {}
            _ => {
                d.status = match c {
                    'h' => {
                        if d.status == DayStatus::Holiday {
                            DayStatus::Worked
                        } else {
                            DayStatus::Holiday
                        }
                    }
                    's' => {
                        if d.status == DayStatus::Sick {
                            DayStatus::Worked
                        } else {
                            DayStatus::Sick
                        }
                    }
                    _ => d.status,
                };
                self.update_display_post_direct_edit(old_status, d, &settings)?;
            }
        }
        Ok(())
    }

    fn update_display_post_direct_edit(
        &mut self,
        old_status: DayStatus,
        d: FlexDay,
        settings: &Settings,
    ) -> Result<(), Error> {
        self.update_display_post_edit(old_status, d, &settings)?;
        self.curses.week_win.mv(self.curses.cur_y_in_week(&d), 0);
        if d.total_minutes() < 0 {
            self.curses.week_win.attron(COLOR_PAIR(1));
        }
        self.curses.print_selected_day(&d);
        if d.total_minutes() < 0 {
            self.curses.week_win.attroff(COLOR_PAIR(1));
        }
        self.curses.week_win.refresh();
        Ok(())
    }

    pub fn change_time(
        &mut self,
        time: NaiveTime,
        field: HourField,
        settings: &Settings,
    ) -> Result<(), Error> {
        let mut d = self.get_current_day().clone();
        match d.status {
            DayStatus::Worked | DayStatus::Half => {
                match field {
                    HourField::Begin => d.start = time,
                    HourField::End => d.end = time,
                }
                self.update_display_post_direct_edit(d.status, d, &settings)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn update_display_post_edit(
        &mut self,
        old_status: DayStatus,
        d: FlexDay,
        settings: &Settings,
    ) -> Result<(), Error> {
        self.days_off.update_days_off(old_status, d);
        let week = self
            .current_month
            .update_day(d)
            .expect("Should find a week");
        self.current_month.update_balance();
        self.current_month.save();
        self.days_off.save()?;
        self.curses
            .print_status(&settings, &self.current_month, &self.days_off);
        self.curses
            .print_week_total(&week, week.total_minutes() < settings.week_goal);
        Ok(())
    }

    pub fn edit_settings(&mut self, mut settings: &mut Settings) -> Result<(), Error> {
        self.curses.open_settings(&settings, &self.days_off);
        let mut cur_idx = 0;
        let mut cur_field = 0;
        let mut done = false;
        self.select_option(cur_idx, cur_field, &settings);
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
                        }
                        Input::KeyDown => {
                            digit_idx = 0;
                            if cur_field <= 5 {
                                cur_idx = (cur_idx + 1) % 6;
                            } else {
                                cur_idx = (cur_idx + 1) % 3;
                            }
                            self.select_option(cur_idx, cur_field, &settings)
                        }
                        Input::KeyLeft => {
                            digit_idx = 0;
                            if cur_field > 0 {
                                cur_field -= 1;
                            } else {
                                cur_field = 6;
                                if cur_idx > 2 {
                                    cur_idx = 2;
                                }
                            }
                            self.select_option(cur_idx, cur_field, &settings)
                        }
                        Input::KeyRight => {
                            digit_idx = 0;
                            cur_field = (cur_field + 1) % 7;
                            if cur_field > 5 {
                                if cur_idx > 2 {
                                    cur_idx = 2;
                                }
                            }
                            self.select_option(cur_idx, cur_field, &settings)
                        }
                        Input::Character(c) if c >= '0' && c <= '9' => {
                            self.manage_option_edition(
                                cur_idx,
                                cur_field,
                                c,
                                digit_idx,
                                &mut settings,
                            );
                            digit_idx = (digit_idx + 1) % 2;
                            self.select_option(cur_idx, cur_field, &settings)
                        }
                        _ => {}
                    };
                    cur_field = if cur_idx == 5 {
                        if cur_field < 4 {
                            4
                        } else if cur_field > 5 {
                            5
                        } else {
                            cur_field
                        }
                    } else {
                        cur_field
                    };
                    self.select_option(cur_idx, cur_field, &settings)
                }
                None => {}
            }
        }
        settings.save();
        self.days_off.save()?;
        self.curses.close_setting();
        self.init(&settings);
        Ok(())
    }

    fn select_option(&mut self, cur_idx: i32, cur_field: i32, settings: &Settings) {
        self.curses
            .highlight_option(cur_idx, cur_field, &settings, &self.days_off)
    }

    fn manage_option_edition(
        &mut self,
        cur_idx: i32,
        cur_field: i32,
        c: char,
        digit_idx: i32,
        settings: &mut Settings,
    ) {
        if cur_idx < 5 {
            match cur_field {
                sched_field if sched_field <= 5 => {
                    let mut d = settings.week_sched.sched[cur_idx as usize];
                    match sched_field {
                        0 => {
                            d.start = editor::process_digit_input_for_time(
                                d.start,
                                TimeField::Hour,
                                c,
                                digit_idx,
                            )
                        }
                        1 => {
                            d.start = editor::process_digit_input_for_time(
                                d.start,
                                TimeField::Minute,
                                c,
                                digit_idx,
                            )
                        }
                        2 => {
                            d.end = editor::process_digit_input_for_time(
                                d.end,
                                TimeField::Hour,
                                c,
                                digit_idx,
                            )
                        }
                        3 => {
                            d.end = editor::process_digit_input_for_time(
                                d.end,
                                TimeField::Minute,
                                c,
                                digit_idx,
                            )
                        }
                        4 => {
                            d.pause = editor::process_digit_input_for_duration(
                                d.pause,
                                TimeField::Hour,
                                c,
                                digit_idx,
                            )
                        }
                        5 => {
                            d.pause = editor::process_digit_input_for_duration(
                                d.pause,
                                TimeField::Minute,
                                c,
                                digit_idx,
                            )
                        }
                        _ => unreachable!(),
                    };
                    settings.week_sched.sched[cur_idx as usize] = d;
                }
                6 => match cur_idx {
                    0 => {
                        settings.holidays_per_year = editor::process_digit_input_for_number(
                            settings.holidays_per_year,
                            c,
                            digit_idx,
                        );
                    }
                    1 => {
                        self.days_off.holidays_left = editor::process_digit_input_for_number(
                            self.days_off.holidays_left,
                            c,
                            digit_idx,
                        );
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        } else {
            match cur_field {
                4 => {
                    settings.week_goal = editor::process_digit_input_for_duration(
                        settings.week_goal,
                        TimeField::Hour,
                        c,
                        digit_idx,
                    )
                }
                5 => {
                    settings.week_goal = editor::process_digit_input_for_duration(
                        settings.week_goal,
                        TimeField::Minute,
                        c,
                        digit_idx,
                    )
                }
                _ => unreachable!(),
            }
            settings.holiday_duration = settings.week_goal / 5;
        }
    }
}
