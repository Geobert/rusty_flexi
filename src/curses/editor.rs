use timedata::*;
use chrono::{NaiveTime, Weekday, Duration, Timelike};
use std::ops::{Add, Sub};

pub enum TimeField {
    Hour,
    Minute,
}

fn scroll_status(d: &mut FlexDay, up: bool) {
    let wd = d.weekday().expect("should have weekday");
    d.status = if up {
        match d.status {
            DayStatus::Worked | DayStatus::Holiday => DayStatus::Worked,
            DayStatus::Half => DayStatus::Holiday,
            DayStatus::Sick => DayStatus::Half,
            DayStatus::Weekend => DayStatus::Worked,
        }
    } else {
        match d.status {
            DayStatus::Worked => {
                match wd {
                    Weekday::Sat | Weekday::Sun => DayStatus::Weekend,
                    _ => DayStatus::Holiday,
                }
            }
            DayStatus::Holiday => DayStatus::Half,
            DayStatus::Half => DayStatus::Sick,
            DayStatus::Sick => DayStatus::Sick,
            DayStatus::Weekend => DayStatus::Weekend,
        }
    }
}

fn add_to_hour(time: NaiveTime, up: bool, nb: i64) -> NaiveTime {
    let duration = Duration::hours(nb);
    if up {
        time.add(duration)
    } else {
        time.sub(duration)
    }
}

fn add_to_minute(time: NaiveTime, up: bool, nb: i64) -> NaiveTime {
    let duration = Duration::minutes(nb);
    if up {
        time.add(duration)
    } else {
        time.sub(duration)
    }
}

pub fn process_key_up_down(cur_field: usize, up: bool, mut d: &mut FlexDay) {
    match cur_field {
        0 => {
            scroll_status(&mut d, up);
        }
        1 => {
            d.start = add_to_hour(d.start, up, 1);
        }
        2 => {
            d.start = add_to_minute(d.start, up, 1);
        }
        3 => {
            d.end = add_to_hour(d.end, up, 1);
        }
        4 => {
            d.end = add_to_minute(d.end, up, 1);
        }
        5 => d.pause += if up { 60 } else { -60 },
        6 => d.pause += if up { 1 } else { -1 },
        _ => unreachable!(),
    }
}

fn edit_2nd_digit_hour(time: NaiveTime, digit: u32) -> NaiveTime {
    match time.hour() {
        1 => {
            time.with_hour(time.hour() * 10 + digit).expect(&format!(
                "something wrong while with_hour with {}",
                time.hour() * 10 + digit
            ))
        }
        2 if digit <= 3 => {
            time.with_hour(time.hour() * 10 + digit).expect(&format!(
                "something wrong while with_hour with {}",
                time.hour() * 10 + digit
            ))
        }
        _ => time,
    }
}

fn edit_2nd_digit_minute(time: NaiveTime, digit: u32) -> NaiveTime {
    if time.minute() <= 5 {
        time.with_minute(time.minute() * 10 + digit).expect(
            &format!(
                "something wrong while with_minute with {}",
                time.minute() * 10 +
                    digit
            ),
        )
    } else {
        time
    }
}

pub fn process_digit_input_for_time(
    time: NaiveTime,
    field: TimeField,
    c: char,
    digit_idx: i32,
) -> NaiveTime {
    let digit = c.to_digit(10);
    if digit_idx == 0 {
        match digit {
            Some(digit) => {
                match field {
                    TimeField::Hour => {
                        time.with_hour(digit).expect(&format!(
                            "something wrong while with_hour with {}",
                            digit
                        ))
                    }
                    TimeField::Minute => {
                        time.with_minute(digit).expect(&format!(
                            "something wrong while with_minute with {}",
                            digit
                        ))
                    }
                }
            }
            None => {
                match field {
                    TimeField::Hour => {
                        let t = time.hour() / 10;
                        time.with_hour(t).expect(&format!(
                            "something wrong while with_hour with {}",
                            t
                        ))
                    }
                    TimeField::Minute => {
                        let t = time.minute() / 10;
                        time.with_minute(t).expect(&format!(
                            "something wrong while with_minute with {}",
                            t
                        ))
                    }
                }
            }
        }
    } else {
        match digit {
            Some(digit) => {
                match field {
                    TimeField::Hour => edit_2nd_digit_hour(time, digit),
                    TimeField::Minute => edit_2nd_digit_minute(time, digit),
                }
            }
            None => {
                match field {
                    TimeField::Hour => time.with_hour(0).unwrap(),
                    TimeField::Minute => time.with_minute(0).unwrap(),
                }
            }
        }
    }
}

pub fn process_digit_input_for_duration(
    duration: i64,
    field: TimeField,
    c: char,
    digit_idx: i32,
) -> i64 {
    let digit = c.to_digit(10);
    let nb_hours = duration / 60;
    let nb_hours_in_min = nb_hours * 60;
    let min_left = duration - nb_hours_in_min;
    match digit {
        Some(digit) => {
            let digit64 = digit as i64;
            if digit_idx == 0 {
                match field {
                    TimeField::Hour => digit64 * 60 + min_left,
                    TimeField::Minute => nb_hours_in_min + digit64,
                }
            } else {
                match field {
                    TimeField::Hour => nb_hours_in_min * 10 + digit64 * 60 + min_left,
                    TimeField::Minute => {
                        if min_left <= 5 {
                            nb_hours_in_min + min_left * 10 + digit64
                        } else {
                            duration
                        }
                    }
                }
            }
        }
        None => {
            match field {
                TimeField::Hour => (nb_hours / 10) * 60 + min_left,
                TimeField::Minute => nb_hours_in_min + min_left / 10,
            }
        }
    }
}

pub fn process_digit_input_for_number(nb: f32, c: char, digit_idx: i32) -> f32 {
    let digit = match c.to_digit(10) {
        Some(d) => d,
        None => return nb,
    };
    if digit_idx == 0 {
        digit as f32
    } else {
        nb * 10.0 + digit as f32
    }
}

pub fn process_digit_input(cur_field: usize, c: char, digit_idx: i32, d: &mut FlexDay) {
    match cur_field {
        0 => unreachable!(),
        1 => {
            d.start = process_digit_input_for_time(d.start, TimeField::Hour, c, digit_idx);
        }
        2 => {
            d.start = process_digit_input_for_time(d.start, TimeField::Minute, c, digit_idx);
        }
        3 => {
            d.end = process_digit_input_for_time(d.end, TimeField::Hour, c, digit_idx);
        }
        4 => {
            d.end = process_digit_input_for_time(d.end, TimeField::Minute, c, digit_idx);
        }
        5 => {
            d.pause = process_digit_input_for_duration(d.pause, TimeField::Hour, c, digit_idx);
        }
        6 => {
            d.pause = process_digit_input_for_duration(d.pause, TimeField::Minute, c, digit_idx);
        }
        _ => unreachable!(),
    }
}
