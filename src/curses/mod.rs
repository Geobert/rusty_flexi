pub use self::navigator::Navigator;

mod navigator;

use pancurses::*;
use timedata::{FlexWeek, FlexDay, FlexMonth, month_to_string, weekday_to_string};
use settings::Settings;
use chrono::{Datelike, NaiveDate};

pub struct Curses<'a> {
    main_win: &'a Window,
    week_win: Window,
    stat_win: Window,
}

impl<'a> Curses<'a> {
    pub fn new(window: &'a Window) -> Curses {
        Curses {
            main_win: window,
            week_win: window.subwin(11, 48, 1, 2).expect("Week window creation failed"),
            stat_win: window.subwin(11, 25, 1, 51).expect("Status window creation failed")
        }
    }

    pub fn print_week_header(&self, month: u32) {
        let month_str = month_to_string(month);
        self.week_win.mv(0, 0);
        self.week_win.clrtoeol();
        self.week_win.mvprintw(0, (48 / 2 - month_str.len() / 2) as i32, &format!("{}", month_str));
    }

    pub fn print_week(&self, week: &FlexWeek, today: &NaiveDate) {
        println!("print_week : {}", today);
        let mut y = 2;
        self.week_win.mv(y, 0);
        for d in &week.days {
            if d.date.expect("No date in day").day() == today.day() {
                self.week_win.attron(A_BOLD);
                self.week_win.printw(&d.to_string());
                self.week_win.attroff(A_BOLD);
            } else {
                self.week_win.printw(&d.to_string());
            }
            y += 1;
            self.week_win.mv(y, 0);
        }
        self.week_win.printw(&week.total_str());
        self.week_win.refresh();
    }


    pub fn print_prompt(&self, day: &FlexDay) {
        self.main_win.mvprintw(self.week_win.get_max_y() + 1, 2,
                               &format!("Editing \"{} {:02}/{:02}\" (\"?\" for help).",
                                        match day.weekday() {
                                            Some(wd) => weekday_to_string(wd),
                                            None => "???".to_string()
                                        },
                                        match day.date {
                                            Some(date) => date.day(),
                                            None => 0
                                        },
                                        match day.date {
                                            Some(date) => date.month(),
                                            None => 0
                                        }));
        self.main_win.mvprintw(self.week_win.get_max_y() + 2, 2, "> ");
    }

    pub fn delete_prompt(&self) {
        self.main_win.mv(self.week_win.get_max_y() + 1, 2);
        self.main_win.clrtoeol();
        self.main_win.mv(self.week_win.get_max_y() + 2, 2);
        self.main_win.clrtoeol();
    }

    pub fn manage_edit(&self, d: &FlexDay, m: &FlexMonth) {
        let mut done = false;

        while !done {
            self.main_win.nodelay(true);
            half_delay(50);
            let ch = self.main_win.getch();
            nocbreak(); // Reset the halfdelay() value
            cbreak();

            match ch {
                Some(c) => {
                    match c {
                        Input::Character('\u{8}') => {
                            if self.main_win.get_cur_x() > 4 {
                                self.main_win.mv(self.main_win.get_cur_y(),
                                                 self.main_win.get_cur_x() - 1);
                                self.main_win.delch();
                            }
                        }
                        Input::Character('\x1B') => done = true,
                        Input::Character(c) => {
                            println!("{:?}", c);
                            self.main_win.addch(c);
                        }
                        _ => { println!("unknown") }
                    }
                }
                None => {}
            }
        }
    }

    pub fn print_status(&self, settings: &Settings, m: &FlexMonth) {
        let start_y = 7;
        self.stat_win.mvprintw(start_y, 0, &format!("Month balance: {:02}:{:02}", m.balance / 60, m.balance - (m.balance / 60) * 60));
//        self.stat_win.mvprintw(start_y + 1, 0, &format!("Holidays left: {}", settings.holidays.left));
        //        self.stat_win.mvprintw(start_y + 2, 0, &format!("Sick days: {}", settings.sick_days));
    }
}
