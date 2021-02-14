use crate::*;
use std::thread::sleep;
use std::time::Duration;

// This is required as if we keep closing and reopening the connection between each test
// the sqlite driver seems to run into some race condition and rows that should have been
// deleted are not.
#[test]
fn test_e2e() {
    let test_connection = init("test-db").unwrap();
    test_connection.clear();
    println!("test_add_and_list_line");
    test_add_and_list_line(&test_connection);
    test_connection.clear();
    println!();
    println!();
    println!();
    println!("test_add_and_list_flex");
    test_add_and_list_flex(&test_connection);
    test_connection.clear();
    println!();
    println!();
    println!();
    println!("test_add_and_list_and_stop_daily_time_override");
    test_add_and_list_and_stop_daily_time_override(&test_connection);
    test_connection.clear();
    println!();
    println!();
    println!();
    println!("test_combination_of_stuff");
    test_combination_of_stuff(&test_connection);
    test_connection.clear();
}

fn test_add_and_list_line(connection: &DbConnection) {
    sleep(Duration::from_secs(1));
    let matches = get_app().get_matches_from(vec!["cli-tt", "add-time", "10:00", "19:00", "2019-11-10", "-b60"]);
    let message = execute_commands(matches, connection).unwrap().get(0).unwrap().to_string();
    println!("message 1: {}", message);
    assert_eq!(message.contains("2019-11-10"), true);
    assert_eq!(message.contains("10:00"), true);
    assert_eq!(message.contains("19:00"), true);
    assert_eq!(message.contains("60 minutes"), true);
    let message2 = execute_commands(get_app().get_matches_from(vec!["cli-tt", "list-time", "-s2019-11-10", "-e2019-11-11"]), connection).unwrap().get(1).unwrap().to_string();
    println!("message 2: {}", message2);
    assert_eq!(message2.contains("2019-11-10"), true);
    assert_eq!(message2.contains("10:00"), true);
    assert_eq!(message2.contains("19:00"), true);
    assert_eq!(message2.contains("60 minutes"), true);
}

fn test_add_and_list_flex(connection: &DbConnection) {
    sleep(Duration::from_secs(1));
    let matches = get_app().get_matches_from(vec!["cli-tt", "add-flex", "30", "2019-11-10", "-c", "Some text here"]);
    let message = execute_commands(matches, &connection).unwrap().get(0).unwrap().to_string();
    println!("message 1: {}", message);
    assert_eq!(message.contains("2019-11-10"), true);
    assert_eq!(message.contains("30"), true);
    assert_eq!(message.contains("Some text here"), true);
    let message2 = execute_commands(get_app().get_matches_from(vec!["cli-tt", "list-flex", "-s2019-11-10", "-e2019-11-11"]), connection).unwrap().get(1).unwrap().to_string();
    println!("message 2: {}", message2);
    assert_eq!(message.contains("2019-11-10"), true);
    assert_eq!(message.contains("30"), true);
    assert_eq!(message.contains("Some text here"), true);
}

fn test_add_and_list_and_stop_daily_time_override(connection: &DbConnection) {
    sleep(Duration::from_secs(1));
    let matches = get_app().get_matches_from(vec!["cli-tt", "add-daily-time-override", "2020-02-14", "360"]);
    let message = execute_commands(matches, &connection).unwrap().get(0).unwrap().to_string();
    println!("message added: {}", message);
    assert_eq!(message.contains(" 2020-02-14 "), true);
    assert_eq!(message.contains(" 360 "), true);
    let message = execute_commands(get_app().get_matches_from(vec!["cli-tt", "list-daily-time-override"]), &connection).unwrap().get(0).unwrap().to_string();
    println!("message list time override: {}", message);
    assert_eq!(message.contains(" 2020-02-14 "), true);
    assert_eq!(message.contains(" 360 "), true);
    assert_eq!(message.contains(" never "), true);
    assert_eq!(message.contains(" 1 "), true);
    let message = execute_commands(get_app().get_matches_from(vec!["cli-tt", "stop-daily-time-override", "1", "2020-02-15"]), &connection).unwrap().get(0).unwrap().to_string();
    println!("message stop time override: {}", message);
    assert_eq!(message.contains(" 2020-02-15."), true);
    assert_eq!(message.contains(" 1 "), true);
    let message = execute_commands(get_app().get_matches_from(vec!["cli-tt", "list-daily-time-override"]), &connection).unwrap().get(0).unwrap().to_string();
    println!("message list time override: {}", message);
    assert_eq!(message.contains(" 2020-02-14 "), true);
    assert_eq!(message.contains(" 360 "), true);
    assert_eq!(message.contains(" 2020-02-15 "), true);
    assert_eq!(message.contains(" 1 "), true);
}

fn test_combination_of_stuff(connection: &DbConnection) {
    sleep(Duration::from_secs(1));
    execute_commands(get_app().get_matches_from(vec!["cli-tt", "add-time", "08:00", "17:00", "2019-11-11", "-b60"]), connection).unwrap(); // +0
    execute_commands(get_app().get_matches_from(vec!["cli-tt", "add-time", "08:30", "17:00", "2019-11-12", "-b60"]), connection).unwrap(); // -30 p
    execute_commands(get_app().get_matches_from(vec!["cli-tt", "add-time", "08:00", "17:00", "2019-11-13", "-b90"]), connection).unwrap(); // -30 p
    execute_commands(get_app().get_matches_from(vec!["cli-tt", "add-time", "08:00", "17:30", "2019-11-14", "-b60"]), connection).unwrap(); // +30 p
    execute_commands(get_app().get_matches_from(vec!["cli-tt", "add-flex", "120", "2019-11-11"]), connection).unwrap(); // +120
    execute_commands(get_app().get_matches_from(vec!["cli-tt", "add-flex", "120", "2019-11-12"]), connection).unwrap(); // +120 p
    let lines = execute_commands(get_app().get_matches_from(vec!["cli-tt", "report", "-s2019-11-12", "-e2019-11-14"]), connection).unwrap();
    for line in &lines {
        println!("{}", line);
    }
    println!();
    assert_eq!(lines.last().unwrap().contains("period: 1.50 hours"), true);
    assert_eq!(lines.last().unwrap().contains("spend: 3.50"), true);
    execute_commands(get_app().get_matches_from(vec!["cli-tt", "add-daily-time-override", "2019-11-13", "360"]), connection).unwrap(); // Changes the 13th time to be +90 and 14th to be +150 instead (total change + 4 hours)
    let lines = execute_commands(get_app().get_matches_from(vec!["cli-tt", "report", "-s2019-11-12", "-e2019-11-14"]), connection).unwrap();
    for line in &lines {
        println!("{}", line);
    }
    println!();
    assert_eq!(lines.last().unwrap().contains("period: 5.50 hours"), true);
    assert_eq!(lines.last().unwrap().contains("spend: 7.50"), true);
    execute_commands(get_app().get_matches_from(vec!["cli-tt", "stop-daily-time-override", "1", "2019-11-14"]), connection).unwrap(); // Changes the 14 time back to +30 (total time reduced by 2 hours)
    let lines = execute_commands(get_app().get_matches_from(vec!["cli-tt", "report", "-s2019-11-12", "-e2019-11-14"]), connection).unwrap();
    for line in &lines {
        println!("{}", line);
    }
    println!();
    assert_eq!(lines.last().unwrap().contains("period: 3.50 hours"), true);
    assert_eq!(lines.last().unwrap().contains("spend: 5.50"), true);
}