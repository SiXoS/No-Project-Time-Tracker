
use crate::db::db_manager::{ DateLine, FlexLine };
use chrono::{ DateTime, Local, Date, NaiveDate };
use std::collections::btree_map::{ BTreeMap };

pub fn create_csv_report(time_rows: Vec<DateLine>, flex_rows: Vec<FlexLine>, total_flex_hours: f64) -> Vec<String> {
    let mut flex_for_period = 0.0;
    let mut lines = Vec::new();
    let map = build_map_by_date(time_rows);

    lines.push("Date,Start,End,Break,Flex (minutes),,,Flex for period (hours),Flex total (hours)".to_string());
    for (date, date_lines) in map {
        let flex = calculate_flex(&date_lines);
        flex_for_period += flex as f64 / 60.0;
        let first_line = &date_lines[0];
        lines.push(format!("{},{},{},{},{}", date.format("%Y-%m-%d"), first_line.start.format("%H:%M"), first_line.end.format("%H:%M"), first_line.break_time_minutes, flex));
        for i in 1 .. date_lines.len() {
            lines.push(format!(",{},{},{},", date_lines[i].start.format("%H:%M"), date_lines[i].end.format("%H:%M"), date_lines[i].break_time_minutes));
        }
    }
    for row in &flex_rows {
        flex_for_period += row.flex_minutes as f64 / 60.0;
    }
    append_string_line_or_push_new(&mut lines, 1, format!("{:.2},{:.2}", flex_for_period, total_flex_hours));
    append_string_line_or_push_new(&mut lines, 3, "Date for flex,Minutes reported,Comment".to_string());
    let mut i = 4;
    for row in flex_rows {
        let date: Date<Local> = DateTime::from(row.date).date();
        append_string_line_or_push_new(&mut lines, i, format!("{},{},{}", date, row.flex_minutes, row.comment));
        i += 1;
    }
    lines
}

fn build_map_by_date(time_rows: Vec<DateLine>) -> BTreeMap<NaiveDate, Vec<DateLine>> {
    let mut map: BTreeMap<NaiveDate, Vec<DateLine>> = BTreeMap::new();
    for row in time_rows {
        match map.get_mut(&row.date) {
            None => {
                let mut list = Vec::new();
                let date = row.date;
                list.push(row);
                map.insert(date, list);
            },
            Some(list) => list.push(row)
        };
    }
    map
}

fn append_string_line_or_push_new(rows: &mut Vec<String>, index: usize, to_append: String) {
    while rows.len() < index {
        rows.push("".to_string())
    }
    if rows.len() == index {
        rows.push(format!(",,,,,,,{}", to_append));
    } else {
        rows[index] = format!("{},,,{}", rows[index], to_append);
    }
}

pub fn create_human_friendly_report(time_rows: Vec<DateLine>, flex_rows: Vec<FlexLine>, total_flex_hours: f64, start: DateTime<Local>, end: DateTime<Local>) -> Vec<String> {
    let mut flex_for_period = 0.0;
    let mut lines = Vec::new();
    let map = build_map_by_date(time_rows);
    lines.push(format!("Time entries from {} to {}.", start, end));
    for (date, date_line) in map {
        let flex = calculate_flex(&date_line);
        flex_for_period += flex as f64 / 60.0;
        lines.push(format!("Got {} flex minutes from {}:", flex, date.format("%Y-%m-%d")));
        for date_line in date_line {
            lines.push(format!("Worked from {} to {} with a break of {} minutes", date_line.start.format("%H:%M"), date_line.end.format("%H:%M"), date_line.break_time_minutes))
        }
    }
    lines.push("Manual flex entries:".to_string());
    for row in flex_rows {
        flex_for_period += row.flex_minutes as f64 / 60.0;
        let date: Date<Local> = DateTime::from(row.date).date();
        lines.push(format!("Registered {} minutes of flex at {} with comment: '{}'", row.flex_minutes, date, row.comment))
    }
    lines.push(format!("Flex diff for selected period: {:.2} hours. Total flex to spend: {:.2} hours", flex_for_period, total_flex_hours));
    lines
}

fn calculate_flex(rows_for_date: &Vec<DateLine>) -> i64 {
    let mut sum_minutes = 0;
    for row in rows_for_date {
        sum_minutes += (row.end.timestamp() - row.start.timestamp()) / 60 - row.break_time_minutes as i64
    }
    return sum_minutes - 8 * 60;
}


