#![windows_subsystem = "windows"]
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate chrono;
extern crate pancurses;

mod timedata;
mod settings;
mod savable;
mod curses;

use pancurses::*;
use curses::*;
use chrono::{Duration, NaiveDate, Datelike, NaiveTime, Timelike};
use std::ops::{Add, Sub};
use timedata::{FlexMonth, DayStatus};
use settings::Settings;

fn generate_xmas_holidays(today: NaiveDate, settings: &Settings) {
    {
        let (mut december, from_json) = FlexMonth::load_with_flag(today.year(), 12, &settings);
        if !from_json {
            // newly created month, auto set holiday as we always have 5 days of holidays in december
            let nb_weeks = december.weeks.len();
            let mut week_to_edit = december.weeks[nb_weeks - 1].clone();
            for day in week_to_edit.days.iter_mut() {
                if day.status == DayStatus::Worked {
                    day.status = DayStatus::Holiday;
                }
            }
            december.weeks[nb_weeks - 1] = week_to_edit;
            december.save();
        }
    }
    {
        let (mut january, from_json) = FlexMonth::load_with_flag(today.year(), 01, &settings);
        if !from_json {
            // newly created month, auto set holiday as we always have 2 days of holidays in january
            let mut week_to_edit = january.weeks[0].clone();
            week_to_edit[0].status = DayStatus::Holiday;
            week_to_edit[1].status = DayStatus::Holiday;
            january.weeks[0] = week_to_edit;
            january.save();
        }
    }
}

fn main() {
    timedata::create_data_dir();
    let window = initscr();
    window.keypad(true);
    noecho();
    cbreak();
    start_color();
    curs_set(0);

    init_pair(1, COLOR_RED, COLOR_BLACK);
    let today = chrono::Local::today().naive_local();
    let mut navigator = Navigator::new(today, &window);

    let settings = navigator.settings.clone();
    generate_xmas_holidays(today, &settings);

    navigator.init();

    let mut done = false;
    while !done {
        match window.getch() {
            Some(c) => match c {
                Input::Character('q') | Input::Character('\x1B') => done = true,
                Input::KeyUp => { navigator.select_prev_day(); },
                Input::KeyDown => { navigator.select_next_day(); },
                Input::KeyLeft => { navigator.select_prev_week(); },
                Input::KeyRight => { navigator.select_next_week(); },
                Input::KeyPPage => { navigator.change_month(false); },
                Input::KeyNPage => { navigator.change_month(true); },
                Input::Character('\n') => { navigator.edit_day(); },
                Input::Character(c) if c == 'h' || c == 's' => { navigator.change_status(c); },
                Input::Character('o') => { navigator.edit_settings(); }
                Input::KeyHome => {
                    let today = chrono::Local::today().naive_local();
                    navigator.select_day(today);
                },
                Input::Character(c) if c == 'b' || c == 'e' => {
                    let offset = Duration::minutes(navigator.settings.offset);
                    let t = chrono::Local::now().naive_local().time();
                    let t = NaiveTime::from_hms(t.hour(), t.minute(), 0); // clear seconds
                    navigator.change_time(if c == 'b' { t.sub(offset) } else { t.add(offset) },
                                          if c == 'b' { HourField::Begin } else { HourField::End });
                    navigator.edit_day();
                },
                _ => { println!("unknown: {:?}", c); }
            },
            None => {}
        }
    }
    endwin();
}

