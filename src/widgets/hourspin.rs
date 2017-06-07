#![feature(proc_macro)]
extern crate gtk;

use chrono::{NaiveTime, Duration};

use gtk::prelude::*;
use gtk::{
    WidgetExt,
    ContainerExt,
    EntryExt,
    Adjustment
};

use relm::Widget;
use relm_attributes::widget;

#[derive(Msg, Debug)]
pub enum Msg {
    Changed,
    Display,
}

#[widget]
impl Widget for HourSpin {
    fn model(m: NaiveTime) -> NaiveTime {
        m
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Display => self.format_display(),
            Msg::Changed => self.model = self.model + Duration::minutes(1),
        };
    }

    fn format_display(&mut self) {
        self.spin_btn.set_text(&self.model.format("%H:%M").to_string());
    }

    view! {
        #[name="spin_btn"]
        gtk::SpinButton {
            max_width_chars: 5,
            width_chars: 5,
            max_length: 5,
            numeric: false,
            output => (Msg::Display, Inhibit(true)),
//            value_changed => (Msg::Changed, Inhibit(true)),
        }
    }
}

#[widget]
impl Widget for DurationSpin {
    fn model(m: Duration) -> Duration {
        m
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Display => self.format_display(),
            Msg::Changed => {
                println!("update, self.spin_btn.get_value_as_int() = {:?}", self.spin_btn.get_value_as_int());
                self.model = Duration::minutes(self.spin_btn.get_value_as_int() as i64)
                
            },
        };
    }

    fn format_display(&mut self) {
        println!("format_display, self.spin_btn.get_value_as_int() = {:?}", self.spin_btn.get_value_as_int());
        let minus = self.model.num_hours() * 60;
        self.spin_btn.set_text(&format!("{:02}:{:02}",
                                        self.model.num_hours(), self.model.num_minutes() - minus));
    }

    view! {
        #[name="spin_btn"]
        gtk::SpinButton {
            max_width_chars: 5,
            width_chars: 5,
            max_length: 5,
            numeric: false,
            output => (Msg::Display, Inhibit(false)),
            value_changed => Msg::Changed,
            adjustment: &Adjustment::new(self.model.num_minutes() as f64, 0.0, 720.0, 1.0, 60.0, 0.0),
        }
    }
}