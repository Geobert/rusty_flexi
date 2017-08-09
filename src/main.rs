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

fn main() {
    timedata::create_data_dir();
    let settings = Settings::load();
    let window = initscr();
    window.keypad(true);
    noecho();
    cbreak();
    start_color();
    curs_set(0);

    init_pair(1, COLOR_RED, COLOR_BLACK);

    let curses = Curses::new(&window);
    let today = chrono::Local::today().naive_utc();
    let mut navigator = Navigator::new(today, &curses, &settings);
    navigator.init();
    let mut done = false;
    while !done {
        match window.getch() {
            Some(c) => match c {
                Input::Character('q') | Input::Character('\x1B') => done = true,
                Input::KeyUp => { navigator.select_prev_day(); },
                Input::KeyDown => { navigator.select_next_day(); },
                Input::KeyPPage => { navigator.change_month(false); },
                Input::KeyNPage => { navigator.change_month(true); },
                Input::Character('\n') | Input::KeyRight | Input::KeyLeft => {
                    navigator.manage_edit();
                },
                Input::Character(c) if c == 'h' || c == 's' => { navigator.change_status(c); },
                Input::Character('o') | Input::Character('s') => {}
                _ => {}
            },
            None => {}
        }
    }
    endwin();
}

