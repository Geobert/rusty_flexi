#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate chrono;

mod timedata;
mod settings;

use settings::Settings;

fn main() {
    println!("Hello, world!");
    let settings = Settings::load();
    
}
