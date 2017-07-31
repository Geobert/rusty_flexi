#![feature(proc_macro)]
extern crate gtk;
extern crate futures;
extern crate futures_glib;

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


//#[widget]
//impl Widget for HourSpin {
//    fn model(m: NaiveTime) -> NaiveTime {
//        m
//    }
//
//    fn update(&mut self, event: Msg) {
//        match event {
//            Msg::Display => self.format_display(),
//            Msg::Changed(_) => self.model = self.model + Duration::minutes(1),
//            Msg::Input => (),
//        };
//    }
//
//    fn format_display(&mut self) {
//        self.spin_btn.set_text(&self.model.format("%H:%M").to_string());
//    }
//
//    view! {
//        #[name="spin_btn"]
//        gtk::SpinButton {
//            max_width_chars: 5,
//            width_chars: 5,
//            max_length: 5,
//            numeric: false,
//            output => (Msg::Display, Inhibit(true)),
////            value_changed => (Msg::Changed, Inhibit(true)),
//        }
//    }
//}
#[derive(Msg, Debug)]
pub enum Msg {
    Changed,
    Display,
    Input,
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
                let v = self.spin_btn.get_value_as_int() as i64;
                println!("update, self.spin_btn.get_value_as_int() = {:?}", v);
                self.model = Duration::minutes(v)
            },
            Msg::Input => self.parse_display(),
        };
    }

    fn parse_display(&mut self) -> Option<Result<f64, ()>> {
        let t = &self.spin_btn.get_text().unwrap();
        let hours_in_min = t[0..1].parse::<i32>().unwrap() * 60;
        let min = t[2..3].parse::<i32>().unwrap() + hours_in_min;
        Some(Ok(min as f64))
    }

    fn format_display(&mut self) {
        let v = self.spin_btn.get_value_as_int() as i64;
        println!("format_display, self.spin_btn.get_value_as_int() = {:?}", v);
        let hours_as_min = self.model.num_hours() * 60;
        self.spin_btn.set_text(&format!("{:02}:{:02}",
                                        self.model.num_hours(),
                                        self.model.num_minutes() - hours_as_min));
    }

    view! {
        #[name="spin_btn"]
        gtk::SpinButton {
            max_width_chars: 5,
            width_chars: 5,
            max_length: 5,
            numeric: false,
            output => (Msg::Display, Inhibit(false)),
            input => (Msg::Input, None),
            value_changed => Msg::Changed,
            adjustment: &Adjustment::new(self.model.num_minutes() as f64, 0.0, 720.0, 1.0, 60.0, 0.0),
        }
    }
}