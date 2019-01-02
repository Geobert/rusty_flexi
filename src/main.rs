#![cfg_attr(not(test), windows_subsystem = "windows")]

mod curses;
mod savable;
mod settings;
mod timedata;

// use crate::curses::settingseditor;
use crate::curses::*;
use crate::settings::Settings;
use crate::timedata::FlexMonth;
use chrono::Datelike;
use failure::Error;
use pancurses::*;

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
    navigator.main_loop(&window, &mut settings)?;
    endwin();
    Ok(())
}
