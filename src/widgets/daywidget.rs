#![feature(proc_macro)]
extern crate gtk;

use chrono::{Datelike, Duration};
use timedata::{FlexDay, weekday_to_string};
use super::hourspin::{HourSpin, DurationSpin};

use gtk::prelude::*;
use gtk::{
    OrientableExt,
    EntryExt,
    ComboBoxExt,
    Justification,
};

use gtk::Orientation::Horizontal;
use relm::Widget;
use relm_attributes::widget;

#[derive(Msg)]
pub enum Msg {
    StatusChanged,
    StartChanged,
    EndChanged,
    BreakChanged
}

#[widget]
impl Widget for DayWidget {
    fn model(model: FlexDay) -> FlexDay {
        model
    }

    fn init_view(&mut self) {
        self.status_cbx.append_text("N");
        self.status_cbx.append_text("H");
        self.status_cbx.append_text("h");
        self.status_cbx.append_text("W");
        self.status_cbx.append_text("S");
        self.status_cbx.set_active(0);

        let width = 35;
        self.day_lbl.set_size_request(width, -1);
        self.date_lbl.set_size_request(width, -1);
        self.hours_edt.set_size_request(width, -1);


    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::StatusChanged => {}
            Msg::StartChanged => {}
            Msg::EndChanged => {}
            Msg::BreakChanged => {}
        }
    }

    view! {
        gtk::Box {
            margin_top: 5,
            orientation: Horizontal,
            spacing: 8,
            #[name="status_cbx"]
            gtk::ComboBoxText {
                margin_left: 10,

            },
            #[name="day_lbl"]
            gtk::Label {
                justify: Justification::Left,
                xalign: 0.0,
                markup: &*match self.model.weekday() {
                    Some(wd) => format!("<span size='medium'>{}</span>", weekday_to_string(wd)),
                    None => format!("<span size='medium'>{}</span>","".to_string())
                }
            }
            #[name="date_lbl"]
            gtk::Label {
                justify: Justification::Left,
                xalign: 0.0,
                markup: &*match self.model.date {
                    Some(date) => format!("<span size='medium'>{:02}/{:02}</span>", date.day(), date.month()),
                    None => format!("<span size='medium'>{}</span>", "".to_string())
                }
            }
            #[name="start_spin"]
            HourSpin(self.model.start),
            #[name="end_spin"]
            HourSpin(self.model.end),
            #[name="break_spin"]
            DurationSpin(Duration::minutes(self.model.pause)),
            #[name="hours_edt"]
            gtk::Entry {
                max_width_chars: 5,
                width_chars: 5,
                max_length: 5
            }
        }
    }
}