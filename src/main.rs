#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate chrono;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

use gtk::{
    WindowExt,
    ButtonExt,
    Inhibit,
    WidgetExt,
    OrientableExt,
    ContainerExt,
    Label
};
use gtk::Orientation::Vertical;
use relm::Widget;
use relm_attributes::widget;

use self::Msg::*;

mod timedata;
mod settings;
mod savable;
mod widgets;

use widgets::DayWidget;
use timedata::FlexDay;
use timedata::FlexWeek;
use timedata::FlexMonth;
use settings::Settings;

#[derive(Msg)]
pub enum Msg {
    Quit
}

#[widget]
impl Widget for Win {
    fn model(model: FlexMonth) -> FlexMonth {
        model
    }

    fn update(&mut self, event: Msg, model: &mut FlexMonth) {
        match event {
            Msg::Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            property_default_height: 650,
            property_default_width: 1000,
            title: "Rusty Flexi",
            #[name="main_box"]
            gtk::Box {
                orientation: Vertical,
                DayWidget(model.weeks[0].days[1]),
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    timedata::create_data_dir();
    let settings = Settings::load();
    let month = FlexMonth::new(2017, 05, &settings);

    Win::run(month).unwrap();
}
