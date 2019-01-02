#![cfg_attr(not(test), windows_subsystem = "windows")]

mod curses;
mod savable;
mod settings;
mod timedata;

// use crate::curses::settingseditor;
use crate::curses::*;
use crate::settings::Settings;
use crate::timedata::FlexMonth;
use chrono::{Datelike, Duration, NaiveTime, Timelike};
use failure::Error;
use pancurses::*;
use std::ops::{Add, Sub};

fn generate_xmas_holidays(year: i32, settings: &Settings) {
    FlexMonth::load(year, 12, &settings);
    FlexMonth::load(year, 01, &settings);
}

fn main() -> Result<(), Error> {
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
    let (mut settings, need_edit_settings) = if let Some(settings) = Settings::load() {
        (settings, false)
    } else {
        (Settings::default(), true)
    };
    let mut navigator = Navigator::new(today, &window, &settings);

    generate_xmas_holidays(today.year(), &settings);

    if need_edit_settings {
        settingseditor::edit_settings(
            &mut navigator.curses,
            &mut settings,
            &mut navigator.days_off,
        )?;
    }
    navigator.init(&settings);

    let mut done = false;
    while !done {
        match window.getch() {
            Some(c) => {
                match c {
                    Input::Character('q') | Input::Character('\x1B') => done = true,
                    Input::KeyUp => {
                        navigator.select_prev_day(&settings);
                    }
                    Input::KeyDown => {
                        navigator.select_next_day(&settings);
                    }
                    Input::KeyLeft => {
                        navigator.select_prev_week(&settings);
                    }
                    Input::KeyRight => {
                        navigator.select_next_week(&settings);
                    }
                    Input::KeyPPage => {
                        navigator.change_month(Direction::Previous, &settings);
                    }
                    Input::KeyNPage => {
                        navigator.change_month(Direction::Next, &settings);
                    }
                    Input::Character('\n') => {
                        navigator.edit_day(&settings)?;
                    }
                    Input::Character(c) if c == 'h' || c == 's' => {
                        navigator.change_status(c, &settings)?;
                    }
                    Input::Character('o') => {
                        settingseditor::edit_settings(
                            &mut navigator.curses,
                            &mut settings,
                            &mut navigator.days_off,
                        )?;
                    }
                    Input::KeyHome => {
                        let today = chrono::Local::today().naive_local();
                        navigator.select_day(today, &settings);
                    }
                    Input::Character(c) if c == 'b' || c == 'e' => {
                        let today = chrono::Local::today().naive_local();
                        navigator.select_day(today, &settings);
                        let offset = Duration::minutes(if c == 'b' {
                            settings.offsets.entry
                        } else {
                            settings.offsets.exit
                        });
                        let t = chrono::Local::now().naive_local().time();
                        let t = NaiveTime::from_hms(t.hour(), t.minute(), 0); // clear seconds
                        navigator.change_time(
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
                            &settings,
                        )?;
                        navigator.edit_day(&settings)?;
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
    Ok(())
}
