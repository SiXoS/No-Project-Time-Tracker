extern crate rusqlite;
extern crate chrono;

use rusqlite::{Connection, Error, NO_PARAMS, Rows, params, Error::QueryReturnedNoRows};
use rusqlite::types::{Null};
use chrono::{DateTime, Local, TimeZone, Date, NaiveDate};
use std::result::*;
use std::option::Option::Some;
use std::path::Path;

const DB_VERSION: i8 = 1;

pub struct DbConnection {
    connection: Connection
}

pub struct DateLine {
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub break_time_minutes: i32,
    pub date: NaiveDate
}

pub struct FlexLine {
    pub date: DateTime<Local>,
    pub flex_minutes: i32,
    pub comment: String
}

pub struct DailyTimeOverrideLine {
    pub id: i32,
    pub start: NaiveDate,
    pub end: Option<NaiveDate>,
    pub minutes_of_work: i32
}

pub fn create_connection<P: AsRef<Path>>(path: P) -> Result<DbConnection, Error> {
    return Ok(DbConnection{ connection: Connection::open(path)?});
}

impl DbConnection {
    pub fn create_tables(&self) -> Result<(), Error> {
        self.connection.execute("CREATE TABLE IF NOT EXISTS time (\
            id INTEGER PRIMARY KEY,\
            start INTEGER NOT NULL,\
            end INTEGER NOT NULL,\
            date TEXT NOT NULL,
            breakTimeMinutes INTEGER NOT NULL\
            )", NO_PARAMS)?;
        self.connection.execute("CREATE TABLE IF NOT EXISTS flex (\
            id INTEGER PRIMARY KEY,\
            flexMinutes INTEGER NOT NULL,\
            date INTEGER NOT NULL,\
            comment TEXT)", NO_PARAMS)?;
        self.connection.execute("CREATE TABLE IF NOT EXISTS version (\
            version INTEGER NOT NULL\
        )", NO_PARAMS)?;
        self.connection.execute("CREATE TABLE IF NOT EXISTS dailyTime (\
            id INTEGER PRIMARY KEY,\
            startDate TEXT NOT NULL,\
            endDate TEXT,\
            minutesOfWork INTEGER NOT NULL\
        )", NO_PARAMS)?;
        self.init_version()?;
        return Ok(());
    }

    fn init_version(&self) -> Result<(), Error> {
        match self.connection.query_row("SELECT version FROM version", NO_PARAMS, |row| row.get::<usize, i8>(0)) {
            Err(QueryReturnedNoRows) => self.connection.execute("INSERT INTO version(version) VALUES(?)", params![DB_VERSION]).map(|_| ()),
            Ok(version) => if version != DB_VERSION {
                unimplemented!("No migration implemented yet")
            } else {
                Ok(())
            },
            err => err.map(|_| ())
        }
    }

    pub fn insert_time(&self, start: &DateTime<Local>, end: &DateTime<Local>, break_time_minutes: i32) -> Result<(), Error> {
        let mut statement = self.connection.prepare("INSERT INTO time(date, start, end, breakTimeMinutes) \
                                                  VALUES(?,?,?,?)")?;
        statement.execute(params![start.format("%Y-%m-%d").to_string(), start.timestamp(), end.timestamp(), break_time_minutes as i64])?;
        return Ok(());
    }

    pub fn get_num_time_entries(&self) -> Result<i32, Error> {
        self.connection.query_row("SELECT COUNT(*) FROM time WHERE start < ?", params![Local::now().timestamp()], |row| row.get(0))
    }

    pub fn get_date_for_last_entry(&self) -> Result<NaiveDate, Error> {
        self.connection.query_row("SELECT date FROM time WHERE start < ? ORDER BY start DESC LIMIT 1", params![Local::now().timestamp()],
                                  |row| Ok(NaiveDate::parse_from_str(row.get::<usize, String>(0)?.as_str(), "%Y-%m-%d").unwrap()))
    }

    pub fn list_times(&self, from: &DateTime<Local>, to: &DateTime<Local>) -> Result<Vec<DateLine>, Error> {
        let mut statement = self.connection.prepare("SELECT start, end, breakTimeMinutes, date FROM time WHERE start > ? AND end < ? ORDER BY start")?;
        let rows = statement.query(&[from.timestamp(), to.timestamp()])?;
        return DbConnection::extract_time_rows(rows);
    }

    fn extract_time_rows(mut rows: Rows) -> Result<Vec<DateLine>, Error> {
        let mut date_lines: Vec<DateLine> = Vec::new();
        while let Some(row) = rows.next()? {
            let date: String = row.get(3)?;
            date_lines.push(DateLine {
                start: Local.timestamp(row.get(0)?, 0),
                end: Local.timestamp(row.get(1)?, 0),
                break_time_minutes: row.get(2)?,
                date: NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d").expect("Could not parse date from DB.")
            });
        }
        return Ok(date_lines);
    }

    pub fn calculate_flex_hours(&self) -> Result<f64, Error> {
        let flex_seconds_from_time: i32 = self.connection.query_row("SELECT IFNULL(SUM(flexTime.seconds_per_day - (CASE WHEN strftime('%w',flexTime.date) IN ('0','6') THEN 0 ELSE IFNULL(dailyTime.minutesOfWork,8*60) END)*60),0) FROM \
            (SELECT SUM(end - start - (breakTimeMinutes*60)) as seconds_per_day, date FROM time GROUP BY date) flexTime LEFT JOIN \
            (SELECT minutesOfWork, startDate, endDate FROM dailyTime) dailyTime ON flexTime.date >= dailyTime.startDate AND (dailyTime.endDate IS NULL OR flexTime.date < dailyTime.endDate)",
               NO_PARAMS, |row| row.get(0))?;
        let flex_minutes_from_flex: i32 = self.connection.query_row("SELECT IFNULL(SUM(flexMinutes),0) FROM flex", NO_PARAMS, |row| row.get(0))?;
        Ok(flex_seconds_from_time as f64 / 60.0 / 60.0 + flex_minutes_from_flex as f64 / 60.0)
    }

    pub fn add_flex(&self, flex_minutes: i32, date: &Date<Local>, comment: Option<&str>) -> Result<(), Error> {
        let mut statement = self.connection.prepare("INSERT INTO flex(flexMinutes, date, comment) VALUES(?,?,?)")?;
        match comment {
            Some(comment) => statement.execute(params![flex_minutes.to_string().as_str(), date.and_hms(0,0,0).timestamp().to_string().as_str(), comment]).map(|_| ())?,
            None => statement.execute(params![flex_minutes.to_string().as_str(), date.and_hms(0,0,0).timestamp().to_string().as_str(), &Null]).map(|_| ())?
        }
        Ok(())
    }

    pub fn list_flex(&self, from: &DateTime<Local>, to: &DateTime<Local>) -> Result<Vec<FlexLine>, Error> {
        let mut statement = self.connection.prepare("SELECT flexMinutes, date, comment FROM flex WHERE date >= ? AND date < ? ORDER BY date")?;
        let mut rows = statement.query(&[from.timestamp(), to.timestamp()])?;
        let mut flex_lines: Vec<FlexLine> = Vec::new();
        while let Some(row) = rows.next()? {
            flex_lines.push(FlexLine {
                flex_minutes: row.get(0)?,
                date: Local.timestamp(row.get(1)?, 0),
                comment: row.get(2).unwrap_or("".to_string())
            });
        }
        return Ok(flex_lines);
    }

    pub fn add_daily_time_override(&self, start: &Date<Local>, minutes_of_work: i32) -> Result<(), Error> {
        let mut statement = self.connection.prepare("INSERT INTO dailyTime(startDate, minutesOfWork) VALUES(?,?)")?;
        statement.execute(params![start.format("%Y-%m-%d").to_string(), minutes_of_work])?;
        Ok(())
    }

    pub fn stop_daily_time_override(&self, id: i32, end: &Date<Local>) -> Result<(), Error> {
        let mut statement = self.connection.prepare("UPDATE dailyTime SET endDate=? WHERE id=?")?;
        let result = statement.execute(params![end.format("%Y-%m-%d").to_string(), id])?;
        return if result == 0 {
            Err(Error::QueryReturnedNoRows)
        } else {
            Ok(())
        }
    }

    pub fn list_daily_time_overrides(&self) -> Result<Vec<DailyTimeOverrideLine>, Error> {
        let mut statement = self.connection.prepare("SELECT id, startDate, endDate, minutesOfWork FROM dailyTime")?;
        let mut rows = statement.query(NO_PARAMS)?;
        let mut daily_times: Vec<DailyTimeOverrideLine> = Vec::new();
        while let Some(row) = rows.next()? {
            let start_date: String = row.get(1)?;
            let end_date: Option<String> = row.get(2)?;
            daily_times.push(DailyTimeOverrideLine {
                id: row.get(0)?,
                start: NaiveDate::parse_from_str(start_date.as_str(), "%Y-%m-%d").expect("Could not parse date from DB."),
                end: end_date.map(|date| NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d").expect("Could not parse date from DB.")),
                minutes_of_work: row.get(3)?
            })
        }
        Ok(daily_times)
    }

    pub fn get_daily_time_override_for_date(&self, date: &Date<Local>) -> Result<Option<DailyTimeOverrideLine>, Error> {
        let mut statement = self.connection.prepare("SELECT id, startDate, endDate, minutesOfWork FROM dailyTime WHERE ? >= dailyTime.startDate AND (dailyTime.endDate IS NULL OR ? < dailyTime.endDate)")?;
        let date_string = date.format("%Y-%m-%d").to_string();
        let mut rows = statement.query(params![date_string, date_string])?;
        match rows.next()? {
            Some(row) => {
                let start_date: String = row.get(1)?;
                let end_date: Option<String> = row.get(2)?;
                Ok(Some(DailyTimeOverrideLine {
                    id: row.get(0)?,
                    start: NaiveDate::parse_from_str(start_date.as_str(), "%Y-%m-%d").expect("Could not parse date from DB."),
                    end: end_date.map(|date| NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d").expect("Could not parse date from DB.")),
                    minutes_of_work: row.get(3)?
                }))
            }
            _ => Ok(None)
        }
    }

    pub fn clear(&self) {
        self.connection.execute("DELETE FROM time", NO_PARAMS).unwrap();
        self.connection.execute("DELETE FROM flex", NO_PARAMS).unwrap();
        self.connection.execute("DELETE FROM dailyTime", NO_PARAMS).unwrap();
    }

}
/*
trait DbLine {
    fn human_output(&self) -> String;
    fn csv_output(&self) -> String;
}

impl DbLine for DateLine {
    fn human_output(&self) -> String {
        let start: DateTime<Local> = DateTime::from(row.start);
        let end: DateTime<Local> = DateTime::from(row.end);
        let flex = (end.timestamp() - start.timestamp())/60 - row.break_time_minutes as i64 - 8*60;
        return format!("from {} to {} with breaks of {} minutes results in {} flex minutes", start, end, row.break_time_minutes, flex)
    }

    fn csv_output(&self) -> String {
        let start: DateTime<Local> = DateTime::from(row.start);
        let end: DateTime<Local> = DateTime::from(row.end);
        let flex = (end.timestamp() - start.timestamp())/60 - row.break_time_minutes as i64 - 8*60;
        format!("{},{},{},{}", start, end, row.break_time_minutes, flex)
    }
}

impl DbLine for FlexLine {
    
}*/