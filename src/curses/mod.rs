pub use self::navigator::Navigator;

mod navigator;
mod editor;

use pancurses::*;
use timedata::{FlexWeek, FlexDay, FlexMonth, DaysOff, DayStatus, month_to_string};
use settings::Settings;
use chrono::{NaiveDate, Timelike, Datelike, Duration};
use std::collections::HashMap;

pub struct Curses<'a> {
    pub main_win: &'a Window,
    pub week_win: Window,
    stat_win: Window,
    // x coord for each field:
    // Status, Start hours, Start min, End Hours, End Min, Break Hours, Break Min
    pub fields: [i32; 7],
    option_win: Option<Window>,
    sub_option_sched: Option<Window>,
    sub_option_days_off: Option<Window>,
}

impl<'a> Curses<'a> {
    pub fn new(window: &'a Window) -> Curses {
        Curses {
            main_win: window,
            week_win: window.subwin(11, 48, 2, 2).expect("Week window creation failed"),
            stat_win: window.subwin(12, 25, 1, 50).expect("Status window creation failed"),
            fields: [0, 16, 19, 25, 28, 33, 36],
            option_win: None,
            sub_option_days_off: None,
            sub_option_sched: None
        }
    }

    pub fn getch(&self) -> Option<Input> {
        self.main_win.nodelay(true);
        half_delay(50);
        let ch = self.main_win.getch();
        nocbreak(); // Reset the halfdelay() value
        cbreak();
        ch
    }

    pub fn print_week_header(&self, month: u32, year: i32) {
        let month_str = month_to_string(month);
        self.week_win.mv(0, 0);
        self.week_win.clrtoeol();
        self.week_win.mvprintw(0, 48 / 2 - (month_str.len() as i32 + 5) / 2,
                               &format!("{} {}", month_str, year));
    }

    // print week, BOLD on today's line
    pub fn print_week(&self, week: &FlexWeek, today: &NaiveDate) {
        let mut y = 2;
        self.week_win.mv(y, 0);
        for d in &week.days {
            let day_is_today = d.date.expect("No date in day").day() == today.day();
            if d.total_minutes() < 0 {
                // end hour before start, print red
                if day_is_today {
                    self.week_win.attron(A_BOLD);
                }
                self.week_win.attron(COLOR_PAIR(1));
                self.week_win.printw(&d.to_string());
                self.week_win.attroff(COLOR_PAIR(1));
                if day_is_today {
                    self.week_win.attroff(A_BOLD);
                }
            } else if day_is_today {
                // bold for selected day
                self.print_selected_day(&d);
            } else {
                match d.status {
                    DayStatus::Weekend | DayStatus::Sick | DayStatus::Half | DayStatus::Holiday => {
                        self.week_win.attron(A_DIM);
                        self.week_win.printw(&d.to_string());
                        self.week_win.attroff(A_DIM);
                    }
                    _ => { self.week_win.printw(&d.to_string()); }
                };
            }
            y += 1;
            self.week_win.mv(y, 0);
        }
    }

    pub fn print_week_total(&self, week: &FlexWeek, below_minimum: bool) {
        self.week_win.mv(9, 0);
        self.week_win.printw(&format!("{:->40} ", " Total ="));

        if below_minimum {
            self.week_win.attron(COLOR_PAIR(1));
        }
        self.week_win.printw(&week.total_str());
        if below_minimum {
            self.week_win.attroff(COLOR_PAIR(1));
        }
        self.week_win.refresh();
    }

    fn print_time(&self, time: u32, status: DayStatus) {
        match status {
            DayStatus::Worked | DayStatus::Half => self.week_win.printw(&format!("{:02}", time)),
            _ => self.week_win.printw("--"),
        };
    }

    fn print_selected_day(&self, d: &FlexDay) {
        self.week_win.attron(A_BOLD);
        self.week_win.printw(&d.to_string());
        self.week_win.attroff(A_BOLD);
    }

    fn highlight_current_field(&self, cur_field: usize, d: &FlexDay, cur_y: i32) {
        // reset any previous reverse attr
        self.week_win.mv(cur_y, 0);
        self.print_selected_day(&d);

        self.week_win.mv(cur_y, self.fields[cur_field]);
        self.week_win.attron(A_REVERSE);
        match cur_field {
            0 => {
                self.week_win.printw(&d.status_str());
            },
            1 => {
                self.print_time(d.start.hour(), d.status);
            },
            2 => {
                self.print_time(d.start.minute(), d.status);
            },
            3 => {
                self.print_time(d.end.hour(), d.status);
            },
            4 => {
                self.print_time(d.end.minute(), d.status);
            },
            5 => {
                self.print_time((d.pause / 60) as u32, d.status);
            },
            6 => {
                self.print_time((d.pause - (d.pause / 60) * 60) as u32, d.status);
            },
            _ => { unreachable!() },
        }
        self.week_win.attroff(A_REVERSE);
    }

    pub fn print_status(&self, settings: &Settings, m: &FlexMonth, off: &DaysOff) {
        let start_y = 1;
        let pad_x = 2;
        self.stat_win.clear();
        self.stat_win.draw_box(0, 0);
        self.stat_win.attron(A_UNDERLINE);
        let stat_title = format!("{} statistics", month_to_string(m.month));
        let width = self.stat_win.get_max_x();
        self.stat_win.mvprintw(start_y, width / 2 - stat_title.len() as i32 / 2, &stat_title);
        self.stat_win.attroff(A_UNDERLINE);
        let goal = settings.week_goal * m.weeks.len() as i64;
        let total = m.total_minute();
        let sign = if m.balance < 0 { "-" } else { " " };
        self.stat_win.mvprintw(start_y + 2, pad_x,
                               &format!("Target:{: >8}{:02}:{:02}", "",
                                        goal / 60, goal - (goal / 60) * 60));
        self.stat_win.mvprintw(start_y + 3, pad_x,
                               &format!("Total:{: >9}{:02}:{:02}", "",
                                        total / 60, total - (total / 60) * 60));
        self.stat_win.mvprintw(start_y + 4, pad_x,
                               &format!("Balance: "));
        if m.balance < 0 {
            self.stat_win.attron(COLOR_PAIR(1));
        }
        self.stat_win.mvprintw(start_y + 4, pad_x + 15,
                               &format!("{}{:02}:{:02}", sign, (m.balance / 60).abs(),
                                        (m.balance - (m.balance / 60) * 60).abs()));
        if m.balance < 0 {
            self.stat_win.attroff(COLOR_PAIR(1));
        }
        let days_off_title = format!("Days off ({})", m.year);
        self.stat_win.attron(A_UNDERLINE);
        self.stat_win.mvprintw(start_y + 6, width / 2 - days_off_title.len() as i32 / 2,
                               &days_off_title);
        self.stat_win.attroff(A_UNDERLINE);
        self.stat_win.mvprintw(start_y + 8, pad_x, &format!("Holidays left: {: >6}",
                                                            off.holidays_left));
        self.stat_win.mvprintw(start_y + 9, pad_x, &format!("Sick days taken: {: >4}",
                                                            off.sick_days_taken));
        self.stat_win.refresh();
    }

    fn print_settings_title(&self, option: &Window, width: i32) {
        option.mv(1, 0);
        let title = "Settings";
        option.mvprintw(1, (width / 2 - title.len() as i32 / 2) as i32, title);
    }

    pub fn open_settings(&mut self, settings: &Settings, off: &DaysOff) {
        let width = 60;
        let height = 14;
        let option = self.main_win.subwin(height, width, 0, (self.week_win.get_max_x() +
            self.stat_win.get_max_x()) / 2 - width / 2)
            .expect("Error while creating options' window");
        option.overlay(self.main_win);
        option.clear();
        self.print_settings_title(&option, width);
        let beg_y = 3;
        let sub_height = height - beg_y - 1;
        let sched = option.derwin(sub_height, 28, beg_y, 2)
            .expect("Error while creating sched option window");
        let days_off = option.derwin(sub_height, 25, beg_y, sched.get_max_x() + 4)
            .expect("Error while creating days off option window");

        self.sub_option_sched = Some(sched);
        self.sub_option_days_off = Some(days_off);

        self.print_sched(&settings);
        self.print_days_off(&off, &settings);

        option.refresh();
        self.option_win = Some(option);
    }

    pub fn close_setting(&mut self) {
        self.sub_option_sched.take().unwrap().delwin();
        self.sub_option_days_off.take().unwrap().delwin();
        self.option_win.take().unwrap().delwin();
    }

    pub fn print_sched(&mut self, settings: &Settings) {
        let sched = self.sub_option_sched.take().unwrap();
        let mut cur_y = 0;
        let title = "Week hours";
        let half_width = sched.get_max_x() / 2;
        sched.attron(A_UNDERLINE);
        sched.mvprintw(cur_y, half_width - title.len() as i32 / 2, title);
        sched.attroff(A_UNDERLINE);
        cur_y += 2;
        for s in &settings.week_sched.sched {
            sched.mvprintw(cur_y, 0, &s.to_string());
            cur_y += 1;
        }
        cur_y += 1;
        let target = Duration::minutes(settings.week_goal);
        sched.mvprintw(cur_y, 0, &format!("Target per week:      {:02}:{:02}", target.num_hours(),
                                          target.num_minutes() - (target.num_hours() * 60)));
        self.sub_option_sched = Some(sched);
    }

    pub fn print_days_off(&mut self, days_off: &DaysOff, settings: &Settings) {
        let off = self.sub_option_days_off.take().unwrap();
        off.border('\u{2502}', ' ', ' ', ' ', '\u{2502}', ' ', ' ', ' ');

        let mut cur_y = 0;
        let title = "Days Off";
        off.attron(A_UNDERLINE);
        off.mvprintw(cur_y, off.get_max_x() / 2 - title.len() as i32 / 2, title);
        off.attroff(A_UNDERLINE);
        cur_y += 2;
        off.mvprintw(cur_y, 2, &format!("Holidays per year: {: >4}", settings.holidays_per_year));
        cur_y += 1;
        off.mvprintw(cur_y, 2, &format!("Holidays left: {: >8}", days_off.holidays_left));
        cur_y += 1;
        off.mvprintw(cur_y, 2, &format!("Sick days taken: {: >6}", days_off.sick_days_taken));

        self.sub_option_days_off = Some(off);
    }

    pub fn highlight_option(&mut self, cur_idx: i32, cur_field: i32,
                            settings: &Settings, off: &DaysOff) {
        let x_coords: HashMap<i32, i32> = [(0, 7), (1, 10), (2, 16), (3, 19), (4, 24), (5, 27)]
            .iter().cloned().collect();
        let win = self.option_win.take().unwrap();
        win.clear();
        win.border('\u{2551}', '\u{2551}', '\u{2550}', '\u{2550}', '\u{2554}', '\u{2557}',
                   '\u{255A}', '\u{255D}');
        // reset any reverse attr
        self.print_settings_title(&win, win.get_max_x());
        self.print_sched(&settings);
        self.print_days_off(&off, &settings);

        // cur_idx == 5 means target week, special field management
        let y = cur_idx + 5 + if cur_idx == 5 { 1 } else { 0 };
        let x = if cur_field > 5 { 55 } else { x_coords[&cur_field] };
        win.mv(y, x);
        win.attron(A_REVERSE);
        let value = match cur_field {
            6 => {
                match cur_idx {
                    0 => settings.holidays_per_year as f32,
                    1 => off.holidays_left,
                    2 => off.sick_days_taken,
                    _ => unreachable!()
                }
            },
            f if f <= 5 => {
                if cur_idx < 5 {
                    let d = settings.week_sched.sched[cur_idx as usize];
                    match f {
                        0 => d.start.hour() as f32,
                        1 => d.start.minute() as f32,
                        2 => d.end.hour() as f32,
                        3 => d.end.minute() as f32,
                        4 => (d.pause / 60) as f32,
                        5 => (d.pause - (d.pause / 60) * 60) as f32,
                        _ => unreachable!()
                    }
                } else {
                    let t = Duration::minutes(settings.week_goal);
                    if f == 4 {
                        t.num_hours() as f32
                    } else {
                        // f should be 5
                        (t.num_minutes() - t.num_hours() * 60) as f32
                    }
                }
            },
            _ => unreachable!(),
        };
        win.printw(&if value == 0.0 && cur_field > 5 {
            format!("{: >2}", value)
        } else {
            format!("{:02}", value)
        });
        win.attroff(A_REVERSE);
        win.refresh();
        self.option_win = Some(win);
    }
}
