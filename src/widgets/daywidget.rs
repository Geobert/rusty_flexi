#![feature(proc_macro)]
extern crate gtk;

use chrono::{Datelike, Weekday};
use timedata::{FlexDay, weekday_to_string, DayStatus};
use std::default::Default;

use gtk::prelude::*;
use gtk::{
    ButtonExt,
    WidgetExt,
    OrientableExt,
    ContainerExt,
    EntryExt,
    ComboBoxExt,
    ListStore,
    TreeModelExt,
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

#[derive(Clone)]
pub struct StatusModel {
    gtk_model: gtk::ListStore,
}

#[widget]
impl Widget for StatusCombo {
    fn model() -> StatusModel {
        let type_in_col = &[gtk::Type::String];
        let list_model = ListStore::new(type_in_col);
//        list_model.insert_with_values(None, &[0], &[""]);
//        list_model.insert_with_values(None, &[0], &["h"]);
//        list_model.insert_with_values(None, &[0], &["H"]);
//        list_model.insert_with_values(None, &[0], &["W"]);
        list_model.insert_with_values(None, &[0], &[&("S".to_value()) as &ToValue]);
        StatusModel {
            gtk_model: list_model
        }
    }

    fn update(&mut self, event: Msg, model: &mut StatusModel) {
    }

    view! {
        gtk::ComboBox {
            model: &model.gtk_model,
        }
    }
}

#[widget]
impl Widget for DayWidget {
    fn model(model: FlexDay) -> FlexDay {
        model
    }

    fn update(&mut self, event: Msg, model: &mut FlexDay) {
        match event {
            Msg::StatusChanged => {}
            Msg::StartChanged => {}
            Msg::EndChanged => {}
            Msg::BreakChanged => {}
        }
    }

    view! {
        gtk::Box {
            orientation: Horizontal,
            spacing: 6,
            #[name="status_cbx"]
            StatusCombo(),
            #[name="day_lbl"]
            gtk::Label {
                text: &*match model.weekday() {
                    Some(wd) => weekday_to_string(wd),
                    None => "".to_string()
                }
            }
            #[name="date_lbl"]
            gtk::Label {
                text: &*match model.date {
                    Some(date) => format!("{:02}/{:02}", date.day(), date.month()),
                    None => "".to_string()
                }
            }
        }
    }
}