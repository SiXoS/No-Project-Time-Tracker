use crate::parsing_utils::*;
use chrono::{Local, Duration, TimeZone};

#[test]
fn test_day_today(){
    assert_eq!(validators::day_validator("today".to_string()).is_ok(), true);
    assert_eq!(parsers::get_date_from_string("today"), Local::now().date());
}

#[test]
fn test_invalid_day_todays(){
    assert_eq!(validators::day_validator("todays".to_string()).is_ok(), false);
}

#[test]
fn test_day_yesterday(){
    assert_eq!(validators::day_validator("yesterday".to_string()).is_ok(), true);
    assert_eq!(parsers::get_date_from_string("yesterday"), Local::now().date().pred());
}

#[test]
fn test_day_x_days(){
    assert_eq!(validators::day_validator("10d".to_string()).is_ok(), true);
    assert_eq!(parsers::get_date_from_string("10d"), (Local::now() - Duration::days(10)).date());
}

#[test]
fn test_day_date(){
    assert_eq!(validators::day_validator("2019-11-10".to_string()).is_ok(), true);
    assert_eq!(parsers::get_date_from_string("2019-11-10"), Local.ymd(2019, 11, 10));
}

#[test]
fn test_parse_brake_time(){
    assert_eq!(validators::unsigned_minute_validator("59".to_string()).is_ok(), true);
    assert_eq!(parsers::force_parse_integer(Some("59")), 59);
    assert_eq!(validators::unsigned_minute_validator("0".to_string()).is_ok(), true);
    assert_eq!(parsers::force_parse_integer(Some("0")), 0);
    assert_eq!(validators::unsigned_minute_validator("5000".to_string()).is_ok(), true);
    assert_eq!(parsers::force_parse_integer(Some("5000")), 5000);
}

#[test]
fn test_bad_brake_time(){
    assert_eq!(validators::unsigned_minute_validator("10d".to_string()).is_ok(), false);
    assert_eq!(validators::unsigned_minute_validator("-1".to_string()).is_ok(), false);
}

#[test]
fn test_date() {
    assert_eq!(parsers::force_parse_datetime(Some("10:11"), Some("2019-11-10")),
               Local.ymd(2019, 11, 10).and_hms(10, 11, 0));
}