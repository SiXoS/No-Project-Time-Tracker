extern crate chrono;
extern crate clap;
extern crate regex;
extern crate dirs;

mod db;
mod parsing_utils;
mod tests;
mod report_generation;

use rusqlite::Error;
use chrono::{DateTime, Local, Datelike, Date, NaiveDate, Weekday, TimeZone};
use clap::{Arg, App, SubCommand, AppSettings, ArgMatches};
use crate::db::db_manager::DbConnection;
use crate::parsing_utils::*;
use crate::report_generation::*;
use std::process;
use std::io::{self};
use std::env;
use dirs::home_dir;
use std::path::{ Path, PathBuf };
use std::fs;

const DB_LOCATION_ENV: &str = "NPTT_DB_LOCATION";

fn main() {
    let mut location = match env::var(DB_LOCATION_ENV) {
        Err(env::VarError::NotPresent) => None,
        Ok(location) => if location.is_empty() { None } else { Some(PathBuf::from(location)) },
        err => panic!(err.unwrap_err())
    }.unwrap_or(home_dir().expect("No DB location set with environment variable NPTT_DB_LOCATION and no home directory found."));
    if !location.exists() {
        fs::create_dir_all(&location).expect(format!("Could not create DB directory ({}). Create the folder with correct permissions or set NPTT_DB_LOCATION to a different location.", location.as_path().to_str().unwrap()).as_str());
    }
    location.push(".nptt-db");
    let connection = init(location).expect("Could not connect to db.");
    let matches = get_app().get_matches();
    match execute_commands(matches, &connection) {
        Ok(lines) => for line in lines {
            println!("{}", line)
        },
        Err(error) => {
            eprintln!("{}", error);
            process::exit(1);
        }
    }
}

fn get_app<'a, 'b>() -> App<'a, 'b> {
    App::new("No Project Time Tracker")
        .version("0.1")
        .author("Simon Lindhén; Github: SiXoS")
        .about("Track your time in a comfortable environment without silly buttons and pictures! Change DB location with environment variable NPTT_DB_LOCATION.")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("add-time")
            .about("Add a new line in time tracking.")
            .arg(Arg::with_name("start")
                .required(true)
                .index(1)
                .validator(validators::time_validator)
                .help("Time to start the line. 24h format: HH:mm"))
            .arg(Arg::with_name("end")
                .required(true)
                .index(2)
                .validator(validators::time_validator)
                .help("Time to end the line. 24h format: HH:mm"))
            .arg(Arg::with_name("day")
                .required(true)
                .index(3)
                .validator(validators::day_validator)
                .help("The day that the time should be recorded for. Applies for both start and end unless -e is specified. Can be one of: 'today', 'yesterday', 'Xd' (X days ago), 'YYYY-MM-dd'"))
            .arg(Arg::with_name("break-time")
                .short("b")
                .takes_value(true)
                .validator(validators::unsigned_minute_validator)
                .help("Minutes of break time you took (lunch mostly).")))
        .subcommand(SubCommand::with_name("smart-add")
            .about("Will allow you to interactively add time for the previous workday(s) that has no time reported. This can be placed in your .bashrc for example. You will then be requested to add the time for unreported days as soon as you open the terminal. Will not do anything if the previous workday has a report.")
            .arg(Arg::with_name("default start")
                .long("start")
                .short("s")
                .takes_value(true)
                .validator(validators::time_validator)
                .help("Default start time of the day. If this is specified it will be presented as an option during the interactive time report. 24h format: HH:mm"))
            .arg(Arg::with_name("default end")
                .long("end")
                .short("e")
                .takes_value(true)
                .validator(validators::time_validator)
                .help("Default end time of the day. If this is specified it will be presented as an option during the interactive time report. 24h format: HH:mm"))
            .arg(Arg::with_name("default break time")
                .long("break")
                .short("b")
                .takes_value(true)
                .validator(validators::unsigned_minute_validator)
                .help("Default break time in minutes. If this is specified it will be presented as an option during the interactive time report.")))
        .subcommand(SubCommand::with_name("list-time")
            .about("List time tracking lines. Shows current month by default.")
            .arg(Arg::with_name("start-day")
                .short("s")
                .takes_value(true)
                .validator(validators::day_validator)
                .help("From which day to list rows. Requires -e. Can be one of: 'today', 'yesterday', 'Xd' (X days ago), 'YYYY-MM-dd'"))
            .arg(Arg::with_name("end-day")
                .short("e")
                .takes_value(true)
                .validator(validators::day_validator)
                .help("To which day to list rows. Requires -s. Can be one of: 'today', 'yesterday', 'Xd' (X days ago), 'YYYY-MM-dd'")))
        .subcommand(SubCommand::with_name("report")
            .about("Get a time report.")
            .arg(Arg::with_name("start-day")
                .short("s")
                .takes_value(true)
                .validator(validators::day_validator)
                .help("From which day to list rows. Requires -e. Can be one of: 'today', 'yesterday', 'Xd' (X days ago), 'YYYY-MM-dd'"))
            .arg(Arg::with_name("end-day")
                .short("e")
                .takes_value(true)
                .validator(validators::day_validator)
                .help("To which day to list rows. Requires -s. Can be one of: 'today', 'yesterday', 'Xd' (X days ago), 'YYYY-MM-dd'"))
            .arg(Arg::with_name("csv")
                .short("c")
                .help("Generates a csv report to stdout.")))
        .subcommand(SubCommand::with_name("add-flex")
            .about("Add additional flex for occasions that don't coincide with normal condition. For example if you get double flex")
            .arg(Arg::with_name("flex-minutes")
                .takes_value(true)
                .required(true)
                .index(1)
                .validator(validators::signed_minute_validator)
                .help("How much flex you want to add in minutes. Use negative value to take from flex"))
            .arg(Arg::with_name("date")
                .takes_value(true)
                .required(true)
                .index(2)
                .validator(validators::day_validator)
                .help("Which date to register the flex time so that it gets in the correct report"))
            .arg(Arg::with_name("comment")
                .takes_value(true)
                .short("c")
                .help("Why did you add this line? So you can remember later on.")))
        .subcommand(SubCommand::with_name("list-flex")
            .about("List flex lines. Shows current month by default.")
            .arg(Arg::with_name("start-day")
                .short("s")
                .takes_value(true)
                .validator(validators::day_validator)
                .help("From which day to list rows. Requires -e. Can be one of: 'today', 'yesterday', 'Xd' (X days ago), 'YYYY-MM-dd'"))
            .arg(Arg::with_name("end-day")
                .short("e")
                .takes_value(true)
                .validator(validators::day_validator)
                .help("To which day to list rows. Requires -s. Can be one of: 'today', 'yesterday', 'Xd' (X days ago), 'YYYY-MM-dd'")))
}

fn execute_commands(matches: ArgMatches, connection: &DbConnection) -> Result<Vec<String>, String> {
    match matches.subcommand() {
        ("add-time", Some(sub_matches)) => add_line(parsers::force_parse_datetime(sub_matches.value_of("start"), sub_matches.value_of("day")),
                                                    parsers::force_parse_datetime(sub_matches.value_of("end"), sub_matches.value_of("day")),
                                                    parsers::force_parse_integer(sub_matches.value_of("break-time")),
                                                    connection),
        ("list-time", Some(sub_matches)) => if sub_matches.is_present("start-day") || sub_matches.is_present("end-day") {
            list_lines(sub_matches.value_of("start-day").map(parsers::get_date_from_string).ok_or("No -s flag specified with -e.")?.and_hms(0, 0, 0),
                       sub_matches.value_of("end-day").map(parsers::get_date_from_string).ok_or("No -e flag specified with -s.")?.succ().and_hms(0, 0, 0),
                       connection)
        } else {
            list_lines(Local::now().with_day(1).unwrap(), plus_one_month(Local::now().with_day(1).unwrap()), connection)
        },
        ("report", Some(sub_matches)) => if sub_matches.is_present("start-day") || sub_matches.is_present("end-day") {
            report(sub_matches.value_of("start-day").map(parsers::get_date_from_string).ok_or("No -s flag specified with -e.")?.and_hms(0, 0, 0),
                   sub_matches.value_of("end-day").map(parsers::get_date_from_string).ok_or("No -e flag specified with -s.")?.succ().and_hms(0, 0, 0),
                   sub_matches.is_present("csv"),
                   connection)
        } else {
            report(Local::now().with_day(1).unwrap(), plus_one_month(Local::now().with_day(1).unwrap()), sub_matches.is_present("csv"), &connection)
        },
        ("add-flex", Some(sub_matches)) => add_flex(parsers::force_parse_integer(sub_matches.value_of("flex-minutes")),
                                                    parsers::force_parse_date(sub_matches.value_of("date")),
                                                    sub_matches.value_of("comment"),
                                                    &connection),
        ("list-flex", Some(sub_matches)) => if sub_matches.is_present("start-day") || sub_matches.is_present("end-day") {
            list_flex(sub_matches.value_of("start-day").map(parsers::get_date_from_string).ok_or("No -s flag specified with -e.")?.and_hms(0, 0, 0),
                      sub_matches.value_of("end-day").map(parsers::get_date_from_string).ok_or("No -e flag specified with -s.")?.succ().and_hms(0, 0, 0),
                      connection)
        } else {
            list_flex(Local::now().with_day(1).unwrap(), plus_one_month(Local::now().with_day(1).unwrap()), connection)
        },
        ("smart-add", Some(sub_matches)) => smart_add(sub_matches.value_of("default start"), sub_matches.value_of("default end"),
                                                      sub_matches.value_of("default break time"), &connection),
        (command, _) => panic!("Command '{}' is not implemented", command)
    }
}

fn smart_add(default_start: Option<&str>, default_end: Option<&str>, default_break: Option<&str>, connection: &DbConnection) -> Result<Vec<String>, String> {
    let num_time_records = connection.get_num_time_entries().expect("Could not fetch existing time records.");
    if num_time_records == 0 {
        return Err("You cannot use smart-add until you have at least one time entry. Add a record with the add-time command.".to_string());
    }
    let mut last_entry = connection.get_date_for_last_entry().expect("Could not fetch time row.");
    last_entry = last_entry.succ();
    let mut dates_to_report = Vec::new();
    let date: Date<Local> = Local::now().date();
    let today = NaiveDate::from_ymd(date.year(), date.month(), date.day());
    while last_entry < today {
        if last_entry.weekday() != Weekday::Sat && last_entry.weekday() != Weekday::Sun {
            dates_to_report.push(last_entry.clone());
        }
        last_entry = last_entry.succ();
    }
    if dates_to_report.len() == 0 {
        Ok(vec![])
    } else {
        for date in dates_to_report {
            smart_add_date(date, default_start, default_end, default_break, connection)?;
        }
        Ok(vec!["Inserted time entries".to_string()])
    }
}

fn smart_add_date(date: NaiveDate, default_start: Option<&str>, default_end: Option<&str>, default_break: Option<&str>, connection: &DbConnection) -> Result<(), String> {
    println!("Adding time for {}:", date.format("%A %e %B %Y"));
    let start = ask_with_optional_default("When did you start? Or type 'skip' to skip this day altogether.", default_start, |value| if value == "skip" {Ok(())} else { validators::time_validator(value) });
    if start == "skip" {
        println!("Ok, skipping.");
        return Ok(());
    }
    let (start_h, start_m) = parsers::force_parse_time(start);
    let (end_h, end_m) = parsers::force_parse_time(ask_with_optional_default("When did you go home?", default_end, validators::time_validator));
    let break_minutes = parsers::force_parse_integer(Some(ask_with_optional_default("How much breaks, in minutes, did you take?", default_break, validators::signed_minute_validator).as_str()));
    let flex = ((end_h * 60 + end_m) as i32 - (start_h * 60 + start_m) as i32 - break_minutes) - 8 * 60;
    let accepted = ask_with_optional_default(format!("Is this correct? {} from {:02}:{:02} to {:02}:{:02} with breaks of {} minutes which results in {} minutes of flex?", date.format("%A %e %B %Y"), start_h, start_m, end_h, end_m, break_minutes, flex).as_str(),
                                             Some("y"), |_| Ok(()));
    if accepted == "y" || accepted == "Y" {
        add_line(Local.ymd(date.year(), date.month(), date.day()).and_hms(start_h, start_m, 0),
                 Local.ymd(date.year(), date.month(), date.day()).and_hms(end_h, end_m, 0),
                 break_minutes, connection).map(|_| ())
    } else {
        println!("Alright, I'll ask again:");
        smart_add_date(date, default_start, default_end, default_break, connection)
    }
}

fn ask_with_optional_default<F>(question: &str, default: Option<&str>, validator: F) -> String
    where
        F: Fn(String) -> Result<(), String>
{
    println!("{}{}", question, default.map(|start| format!(" [{}]", start)).unwrap_or(String::new()));
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("Could not read from stdin");
    match default {
        None => {
            let mut validation_result = validator(buffer.trim().to_string());
            while buffer.trim().is_empty() || validation_result.is_err() {
                println!("{}", if buffer.trim().is_empty() { "Please provide an answer.".to_string() } else { validation_result.unwrap_err() });
                buffer.clear();
                io::stdin().read_line(&mut buffer).expect("Could not read from stdin");
                validation_result = validator(buffer.trim().to_string());
            }
            buffer.trim().to_string()
        }
        Some(default_value) =>
            if buffer.trim().is_empty() {
                default_value.to_string()
            } else {
                let mut validation_result = validator(buffer.trim().to_string());
                while validation_result.is_err() {
                    println!("{}", validation_result.unwrap_err());
                    buffer.clear();
                    io::stdin().read_line(&mut buffer).expect("Could not read from stdin");
                    validation_result = validator(buffer.trim().to_string());
                }
                buffer.trim().to_string()
            }
    }
}

fn plus_one_month(date: DateTime<Local>) -> DateTime<Local> {
    if date.month() == 12 {
        return date.with_month(1).unwrap().with_year(date.year() + 1).unwrap();
    } else {
        return date.with_month(date.month() + 1).unwrap();
    }
}

fn add_line(start: DateTime<Local>, end: DateTime<Local>, break_time: i32, connection: &DbConnection) -> Result<Vec<String>, String> {
    connection.insert_time(&start, &end, break_time)
        .expect("Could not insert row");
    Ok(singleton_vec(format!("Added line: from {} to {} with breaks of {} minutes.", start, end, break_time)))
}

fn list_lines(start: DateTime<Local>, end: DateTime<Local>, connection: &DbConnection) -> Result<Vec<String>, String> {
    let rows = connection.list_times(&start, &end)
        .expect("Could not retrieve lines");
    let mut lines = Vec::new();
    lines.push(format!("Rows from {} to {}:", start, end));
    for row in rows {
        lines.push(format!("from {} to {} with breaks of {} minutes", row.start, row.end, row.break_time_minutes));
    }
    Ok(lines)
}

fn report(start: DateTime<Local>, end: DateTime<Local>, csv: bool, connection: &DbConnection) -> Result<Vec<String>, String> {
    let rows = connection.list_times(&start, &end)
        .expect("Could not retrieve lines.");
    let total_flex = connection.calculate_flex_hours()
        .expect("Could not calculate flex time.");
    let flex_rows = connection.list_flex(&start, &end)
        .expect("Could not retrieve flex lines.");
    if csv {
        Ok(create_csv_report(rows, flex_rows, total_flex))
    } else {
        Ok(create_human_friendly_report(rows, flex_rows, total_flex, start, end))
    }
}

fn add_flex(flex_time_minutes: i32, date: Date<Local>, comment: Option<&str>, connection: &DbConnection) -> Result<Vec<String>, String> {
    connection.add_flex(flex_time_minutes, &date, comment)
        .expect("Could not insert flex entry");
    Ok(vec![format!("Inserted flex entry for {} minutes at {} with comment '{}'", flex_time_minutes, date, comment.unwrap_or(""))])
}

fn list_flex(start: DateTime<Local>, end: DateTime<Local>, connection: &DbConnection) -> Result<Vec<String>, String> {
    let rows = connection.list_flex(&start, &end)
        .expect("Could not retrieve lines");
    let mut lines = Vec::new();
    lines.push(format!("Rows from {} to {}:", start, end));
    for row in rows {
        lines.push(format!("added {} minutes of flex at {} with comment '{}'", row.flex_minutes, row.date.date(), row.comment));
    }
    Ok(lines)
}

fn init<P: AsRef<Path>>(path: P) -> Result<DbConnection, Error> {
    let connection = db::db_manager::create_connection(path)?;
    connection.create_tables()?;
    return Ok(connection);
}

fn singleton_vec(value: String) -> Vec<String> {
    let mut vec = Vec::new();
    vec.push(value);
    return vec;
}
