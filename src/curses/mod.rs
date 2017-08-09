pub use self::navigator::Navigator;

mod navigator;

use pancurses::*;
use timedata::{FlexWeek, FlexDay, FlexMonth, DaysOff, DayStatus, month_to_string};
use settings::Settings;
use chrono::{NaiveDate, Timelike};

pub struct Curses<'a> {
    pub main_win: &'a Window,
    pub week_win: Window,
    stat_win: Window,
    // x coord for each field:
    // Status, Start hours, Start min, End Hours, End Min, Break Hours, Break Min
    pub fields: [i32; 7],
}

impl<'a> Curses<'a> {
    pub fn new(window: &'a Window) -> Curses {
        Curses {
            main_win: window,
            week_win: window.subwin(11, 48, 1, 2).expect("Week window creation failed"),
            stat_win: window.subwin(11, 25, 1, 53).expect("Status window creation failed"),
            fields: [0, 16, 19, 25, 28, 33, 36],
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
        self.week_win.mvprintw(0, (48 / 2 - (month_str.len() + 5) / 2) as i32, // +5 for space + year
                               &format!("{} {}", month_str, year));
    }

    pub fn print_week(&self, week: &FlexWeek, today: &NaiveDate) {
        let mut y = 2;
        self.week_win.mv(y, 0);
        for d in &week.days {
            let day_is_today = d.date.expect("No date in day") == *today;
            if d.total_minutes() < 0 {
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
        self.week_win.refresh();
    }

    pub fn print_status(&self, settings: &Settings, m: &FlexMonth, off: &DaysOff) {
        let start_y = 0;
        self.stat_win.clear();
        self.stat_win.attron(A_UNDERLINE);
        self.stat_win.mvprintw(start_y, 0, &format!("{} statistics", month_to_string(m.month)));
        self.stat_win.attroff(A_UNDERLINE);
        let goal = settings.week_goal * m.weeks.len() as i64;
        let total = m.total_minute();
        let sign = if m.balance < 0 { "-" } else { " " };
        self.stat_win.mvprintw(start_y + 2, 0,
                               &format!("Target:{: >4}{:02}:{:02}", "",
                                        goal / 60, goal - (goal / 60) * 60));
        self.stat_win.mvprintw(start_y + 3, 0,
                               &format!("Total:{: >5}{:02}:{:02}", "",
                                        total / 60, total - (total / 60) * 60));
        self.stat_win.mvprintw(start_y + 4, 0,
                               &format!("Balance:  "));
        if m.balance < 0 {
            self.stat_win.attron(COLOR_PAIR(1));
        }
        self.stat_win.mvprintw(start_y + 4, 11,
                               &format!("{}{:02}:{:02}", sign, (m.balance / 60).abs(),
                                        (m.balance - (m.balance / 60) * 60).abs()));
        if m.balance < 0 {
            self.stat_win.attroff(COLOR_PAIR(1));
        }
        self.stat_win.attron(A_UNDERLINE);
        self.stat_win.mvprintw(start_y + 6, 0, &format!("Days off ({})", m.year));
        self.stat_win.attroff(A_UNDERLINE);
        self.stat_win.mvprintw(start_y + 8, 0, &format!("Holidays left: {}",
                                                        off.holidays_left));
        self.stat_win.mvprintw(start_y + 9, 0, &format!("Sickdays left: {}", off.sick_days_left));
        self.stat_win.refresh();
    }
}
