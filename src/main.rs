#![windows_subsystem = "windows"]
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate failure;
extern crate glob;
extern crate pancurses;
extern crate serde;
extern crate serde_json;

mod curses;
mod savable;
mod settings;
mod timedata;

use chrono::{Datelike, Duration, NaiveTime, Timelike};
use curses::*;
use pancurses::*;
use settings::Settings;
use std::ops::{Add, Sub};
use timedata::FlexMonth;

fn generate_xmas_holidays(year: i32, settings: &Settings) {
    FlexMonth::load(year, 12, &settings);
    FlexMonth::load(year, 01, &settings);
}

fn main() {
    timedata::create_data_dir();
    let window = initscr();
    window.keypad(true);
    noecho();
    cbreak();
    start_color();
    curs_set(0);
    resize_term(15, 76);

    init_pair(1, COLOR_RED, COLOR_BLACK);
    let today = chrono::Local::today().naive_local();
    let mut navigator = Navigator::new(today, &window).unwrap();

    {
        let settings = navigator.settings.clone();
        generate_xmas_holidays(today.year(), &settings);
    }

    navigator.init();

    let mut done = false;
    while !done {
        match window.getch() {
            Some(c) => {
                match c {
                    Input::Character('q') | Input::Character('\x1B') => done = true,
                    Input::KeyUp => {
                        navigator.select_prev_day();
                    }
                    Input::KeyDown => {
                        navigator.select_next_day();
                    }
                    Input::KeyLeft => {
                        navigator.select_prev_week();
                    }
                    Input::KeyRight => {
                        navigator.select_next_week();
                    }
                    Input::KeyPPage => {
                        navigator.change_month(false);
                    }
                    Input::KeyNPage => {
                        navigator.change_month(true);
                    }
                    Input::Character('\n') => {
                        navigator.edit_day().unwrap();
                    }
                    Input::Character(c) if c == 'h' || c == 's' => {
                        navigator.change_status(c).unwrap();
                    }
                    Input::Character('o') => {
                        navigator.edit_settings().unwrap();
                    }
                    Input::KeyHome => {
                        let today = chrono::Local::today().naive_local();
                        navigator.select_day(today);
                    }
                    Input::Character(c) if c == 'b' || c == 'e' => {
                        let today = chrono::Local::today().naive_local();
                        navigator.select_day(today);
                        let offset = Duration::minutes(navigator.settings.offset);
                        let t = chrono::Local::now().naive_local().time();
                        let t = NaiveTime::from_hms(t.hour(), t.minute(), 0); // clear seconds
                        navigator
                            .change_time(
                                if c == 'b' {
                                    t.sub(offset)
                                } else {
                                    t.add(offset)
                                },
                                if c == 'b' {
                                    HourField::Begin
                                } else {
                                    HourField::End
                                },
                            )
                            .unwrap();
                        navigator.edit_day().unwrap();
                    }
                    _ => {
                        println!("unknown: {:?}", c);
                    }
                }
            }
            None => {}
        }
    }
    endwin();
}
