
extern crate regex;
extern crate chrono;

pub mod validators {
    use regex::Regex;
    pub fn time_validator(to_check: String) -> Result<(), String> {
        let regex = Regex::new("^(2[0-4]|[01][0-9]):([0-6][0-9])$").expect("Invalid regex");
        return if regex.is_match(to_check.as_str()) {
            Ok(())
        } else {
            Err(format!("Specified value '{}' is not a valid time format. It should be in HH:mm.", to_check))
        };
    }

    pub fn unsigned_number_validator(to_check: String) -> Result<(), String> {
        let regex = Regex::new("^([0-9]+)$").expect("Invalid regex");
        return if regex.is_match(to_check.as_str()) {
            Ok(())
        } else {
            Err(format!("Specified value '{}' is not a positive integer.", to_check))
        };
    }

    pub fn signed_minute_validator(to_check: String) -> Result<(), String> {
        let regex = Regex::new("^-?[0-9]+$").expect("Invalid regex");
        return if regex.is_match(to_check.as_str()) {
            Ok(())
        } else {
            Err(format!("Specified value '{}' is not an integer.", to_check))
        };
    }

    pub fn day_validator(to_check: String) -> Result<(), String> {
        let regex = Regex::new("^(today|yesterday|[0-9]+d|[0-9]{4}-[0-9]{2}-[0-9]{2})$").expect("Invalid regex");
        return if regex.is_match(to_check.as_str()) {
            Ok(())
        } else {
            Err(format!("Specified value '{}' is not any of 'today', 'yesterday', an integer followed by 'd' (e.g. 10d), or a valid date (yyyy-mm-dd zero padded).", to_check))
        };
    }
}

pub mod parsers {
    use chrono::{DateTime, Local, Date, TimeZone, Duration};
    use regex::Regex;

    pub fn parse_time(time: Option<&str>) -> Option<(u32, u32)> {
        time.map(|s| s.split(":").collect())
            .map(|splitted: Vec<&str>| (splitted[0].to_string().parse().unwrap(), splitted[1].to_string().parse().unwrap()))
    }

    pub fn force_parse_time(time: String) -> (u32, u32) {
        parse_time(Some(time.as_str())).unwrap()
    }

    pub fn force_parse_datetime(time: Option<&str>, date: Option<&str>) -> DateTime<Local> {
        let date = date.unwrap();
        let (hour, minute) = parse_time(time).unwrap();
        return get_date_from_string(date)
            .and_hms(hour,minute, 0);
    }

    pub fn force_parse_date(date: Option<&str>) -> Date<Local> {
        return get_date_from_string(date.unwrap())
    }

    pub fn get_date_from_string(date_string: &str) -> Date<Local> {
        let minus_days_regex = Regex::new("^([0-9]+)d$").expect("invalid regex");
        let date_regex = Regex::new("^([0-9]{4})-([0-9]{2})-([0-9]{2})$").expect("invalid regex");
        if date_string == "today" {
            return Local::now().date();
        } else if date_string == "yesterday" {
            return Local::now().date().pred();
        } else if minus_days_regex.is_match(date_string) {
            match minus_days_regex.captures(date_string) {
                Some(cap) => return Local::now().date() - Duration::days(cap.get(1).unwrap().as_str().to_string().parse::<i64>().unwrap()),
                None => panic!("First matched and then didn't? ({})", date_string)
            }
        } else if date_regex.is_match(date_string) {
            match date_regex.captures(date_string) {
                Some(cap) => return Local.ymd(cap.get(1).unwrap().as_str().parse::<i32>().unwrap(),
                                              cap.get(2).unwrap().as_str().parse::<u32>().unwrap(),
                                              cap.get(3).unwrap().as_str().parse::<u32>().unwrap()),
                None => panic!("First matched and then didn't? ({})", date_string)
            }
        } else {
            return panic!("unexpected date string {}", date_string);
        }
    }

    pub fn force_parse_integer(break_time_string: Option<&str>) -> i32 {
        return parse_integer(break_time_string).unwrap_or(0);
    }

    pub fn parse_integer(break_time_string: Option<&str>) -> Option<i32> {
        return break_time_string.map(|str| str.to_string().parse::<i32>().unwrap());
    }
}
