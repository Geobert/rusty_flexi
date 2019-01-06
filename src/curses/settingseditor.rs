use super::editor;
use super::editor::TimeField;
use super::Curses;
use crate::settings::Settings;
use crate::timedata::*;
use failure::Error;
use pancurses::Input;

pub fn edit_settings<'a>(
  mut curses: &mut Curses<'a>,
  mut settings: &mut Settings,
  mut days_off: &mut DaysOff,
) -> Result<(), Error> {
  curses.open_settings(&settings, &days_off);
  let mut cur_idx = 0;
  let mut cur_field = 0;
  let mut done = false;
  select_option(&mut curses, cur_idx, cur_field, &settings, &days_off);
  let mut digit_idx = 0;
  while !done {
    match curses.getch() {
      Some(Input::Character('\x1B')) => done = true,
      Some(c) => {
        match c {
          Input::KeyUp => {
            digit_idx = 0;
            if cur_idx <= 0 {
              if cur_field <= 5 {
                cur_idx = 4;
              } else {
                cur_idx = 2;
              }
            } else {
              cur_idx -= 1;
            }
          }
          Input::KeyDown => {
            digit_idx = 0;
            if cur_field <= 5 {
              cur_idx = (cur_idx + 1) % 6;
            } else {
              cur_idx = (cur_idx + 1) % 3;
            }
            select_option(&mut curses, cur_idx, cur_field, &settings, &days_off)
          }
          Input::KeyLeft => {
            digit_idx = 0;
            if cur_field > 0 {
              cur_field -= 1;
            } else {
              cur_field = 6;
              if cur_idx > 2 {
                cur_idx = 2;
              }
            }
            select_option(&mut curses, cur_idx, cur_field, &settings, &days_off)
          }
          Input::KeyRight => {
            digit_idx = 0;
            cur_field = (cur_field + 1) % 7;
            if cur_field > 5 {
              if cur_idx > 2 {
                cur_idx = 2;
              }
            }
            select_option(&mut curses, cur_idx, cur_field, &settings, &days_off)
          }
          Input::Character(c) if c >= '0' && c <= '9' => {
            manage_option_edition(
              cur_idx,
              cur_field,
              c,
              digit_idx,
              &mut settings,
              &mut days_off,
            );
            digit_idx = (digit_idx + 1) % 2;
            select_option(&mut curses, cur_idx, cur_field, &settings, &days_off)
          }
          _ => {}
        };
        cur_field = if cur_idx == 5 {
          if cur_field < 4 {
            4
          } else if cur_field > 5 {
            5
          } else {
            cur_field
          }
        } else {
          cur_field
        };
        select_option(&mut curses, cur_idx, cur_field, &settings, &days_off)
      }
      None => {}
    }
  }
  settings.save();
  days_off.save()?;
  curses.close_setting();
  // self.init(&settings);
  Ok(())
}

fn select_option<'a>(
  curses: &mut Curses<'a>,
  cur_idx: i32,
  cur_field: i32,
  settings: &Settings,
  days_off: &DaysOff,
) {
  curses.highlight_option(cur_idx, cur_field, &settings, &days_off)
}

fn manage_option_edition(
  cur_idx: i32,
  cur_field: i32,
  c: char,
  digit_idx: i32,
  settings: &mut Settings,
  days_off: &mut DaysOff,
) {
  if cur_idx < 5 {
    match cur_field {
      sched_field if sched_field <= 5 => {
        let mut d = settings.week_sched.sched[cur_idx as usize];
        match sched_field {
          0 => {
            d.start = editor::process_digit_input_for_time(d.start, TimeField::Hour, c, digit_idx)
          }
          1 => {
            d.start = editor::process_digit_input_for_time(d.start, TimeField::Minute, c, digit_idx)
          }
          2 => d.end = editor::process_digit_input_for_time(d.end, TimeField::Hour, c, digit_idx),
          3 => d.end = editor::process_digit_input_for_time(d.end, TimeField::Minute, c, digit_idx),
          4 => {
            d.pause =
              editor::process_digit_input_for_duration(d.pause, TimeField::Hour, c, digit_idx)
          }
          5 => {
            d.pause =
              editor::process_digit_input_for_duration(d.pause, TimeField::Minute, c, digit_idx)
          }
          _ => unreachable!(),
        };
        settings.week_sched.sched[cur_idx as usize] = d;
      }
      6 => match cur_idx {
        0 => {
          settings.holidays_per_year =
            editor::process_digit_input_for_number(settings.holidays_per_year, c, digit_idx);
        }
        1 => {
          days_off.holidays_left =
            editor::process_digit_input_for_number(days_off.holidays_left, c, digit_idx);
        }
        _ => unreachable!(),
      },
      _ => unreachable!(),
    }
  } else {
    match cur_field {
      4 => {
        settings.week_goal = editor::process_digit_input_for_duration(
          settings.week_goal,
          TimeField::Hour,
          c,
          digit_idx,
        )
      }
      5 => {
        settings.week_goal = editor::process_digit_input_for_duration(
          settings.week_goal,
          TimeField::Minute,
          c,
          digit_idx,
        )
      }
      _ => unreachable!(),
    }
    settings.holiday_duration = settings.week_goal / 5;
  }
}
