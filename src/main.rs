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

use settings::Settings;
use pancurses::*;
use curses::*;
use timedata::{FlexDay, FlexMonth, DaysOff};
use chrono::Datelike;


fn main() {
    timedata::create_data_dir();
    let settings = Settings::load();
    let window = initscr();
    window.keypad(true);
    noecho();
    cbreak();
    let curses = Curses::new(&window);
    let today = chrono::Local::today().naive_utc();
    let days_off = DaysOff::load(today.year(), &settings);
    let mut navigator = Navigator::new(today, &curses, &settings);
    navigator.init();
    let mut done = false;
    while !done {
        match window.getch() {
            Some(c) => match c {
                Input::Character('q') => done = true,
                Input::KeyUp => { navigator.select_prev_day(); }
                Input::KeyDown => { navigator.select_next_day(); }
                Input::KeyPPage => {
                    navigator.change_month(false);
                }
                Input::KeyNPage => {
                    navigator.change_month(true);
                }
                Input::Character('\n') => {
                    let d = navigator.get_current_day();
                    curses.print_prompt(&d);
                    curses.manage_edit(&d, &navigator.current_month);
                    curses.delete_prompt();
                }
                _ => {}
            },
            None => {}
        }
    }
    endwin();
}

