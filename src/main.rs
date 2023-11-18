use time::macros::format_description;
use time::Date;
use time::Duration;
use time::Time;
//tutorial-read-01.rs
use std::{env, error::Error, ffi::OsString, fs::File, process};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
enum Status {
    Entschuldigt,
    Unentschuldigt,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Record {
    abwesenheitszeit: String,
    name: String,
    status: String,
    #[serde(rename = "Aktualisiert am")]
    aktualisiert: String,
}

#[derive(Debug)]
struct RecordAbsenceInHours {
    abwesenheitszeit: f64,
    name: String,
    status: Status,
    aktualisiert: String,
}

impl RecordAbsenceInHours {
    fn new(abwesenheitszeit: f64, name: String, status: Status, aktualisiert: String) -> Self {
        Self {
            abwesenheitszeit,
            name,
            status,
            aktualisiert,
        }
    }
}

#[derive(Debug, Serialize)]
struct RecordAbsenceSum {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Entschuldigt")]
    abwesenheitszeit_entschuldigt: i32,
    #[serde(rename = "Unentschuldigt")]
    abwesenheitszeit_unentschuldigt: i32,
    #[serde(rename = "Gesamt")]
    abwesenheitszeit_summe: i32,
}

impl RecordAbsenceSum {
    fn new(
        name: String,
        abwesenheitszeit_entschuldigt: i32,
        abwesenheitszeit_unentschuldigt: i32,
    ) -> Self {
        let abwesenheitszeit_summe =
            abwesenheitszeit_entschuldigt + abwesenheitszeit_unentschuldigt;
        Self {
            name,
            abwesenheitszeit_entschuldigt,
            abwesenheitszeit_unentschuldigt,
            abwesenheitszeit_summe,
        }
    }
}

const SCHOOL_HOURS_PER_DAY: f64 = 4.0;
const SCHOOL_HOUR_IN_MINUTES: f64 = 67.0;

fn run() -> Result<(), Box<dyn Error>> {
    let file_path = get_first_arg()?;
    let file = File::open(file_path)?;
    let records_absence_in_hours = convert_absence_in_hours(file)?;
    let records_with_absence_sum = calculate_absence_sum(records_absence_in_hours);

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_path("export.csv")?;
    for record in records_with_absence_sum {
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    Ok(())
}

/// Returns the first positional argument sent to this process. If there are no
/// positional arguments, then this returns an error.
fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}

fn convert_absence_in_hours(file: File) -> Result<Vec<RecordAbsenceInHours>, Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new().delimiter(b';').from_reader(file);
    let mut records_absence_in_hours: Vec<RecordAbsenceInHours> = vec![];
    let abwesenheit_time_format = format_description!("[hour]:[minute]");
    let abwesenheit_date_format = format_description!("[day].[month].[year]");
    for result in rdr.deserialize() {
        let record: Record = result?;
        println!("{:?}", record);
        let abwesenheitszeit: f64;
        let status = if record.status == "entschuldigt" {
            Status::Entschuldigt
        } else {
            Status::Unentschuldigt
        };
        if record.abwesenheitszeit.contains(" - ") {
            if record.abwesenheitszeit.ends_with(")") {
                let start_zeit =
                    Time::parse(&record.abwesenheitszeit[12..17], &abwesenheit_time_format)
                        .unwrap();
                let end_zeit =
                    Time::parse(&record.abwesenheitszeit[20..25], &abwesenheit_time_format)
                        .unwrap();
                let duration: Duration = end_zeit - start_zeit;
                abwesenheitszeit = duration.whole_minutes() as f64 / SCHOOL_HOUR_IN_MINUTES;
            } else {
                let start_datum =
                    Date::parse(&record.abwesenheitszeit[0..10], &abwesenheit_date_format)?;
                let end_datum =
                    Date::parse(&record.abwesenheitszeit[13..23], &abwesenheit_date_format)?;
                let duration: Duration = end_datum - start_datum;
                let duration_in_days = duration.whole_days() as f64 + 1.0;
                let duration_in_hours = duration_in_days * SCHOOL_HOURS_PER_DAY;
                abwesenheitszeit = duration_in_hours;
            }
        } else {
            abwesenheitszeit = SCHOOL_HOURS_PER_DAY;
        }
        let record_absence_in_hours =
            RecordAbsenceInHours::new(abwesenheitszeit, record.name, status, record.aktualisiert);
        records_absence_in_hours.push(record_absence_in_hours);
    }
    Ok(records_absence_in_hours)
}

fn calculate_absence_sum(records: Vec<RecordAbsenceInHours>) -> Vec<RecordAbsenceSum> {
    let mut names: Vec<String> = records.iter().map(|r| r.name.clone()).collect();
    names.sort();
    names.dedup();
    let mut records_with_absence_sum: Vec<RecordAbsenceSum> = vec![];
    for name in names {
        let sum_entschuldigt: f64 = records
            .iter()
            .filter(|r| r.name == name && r.status == Status::Entschuldigt)
            .fold(0.0, |acc, r| acc + r.abwesenheitszeit);
        let sum_unentschuldigt: f64 = records
            .iter()
            .filter(|r| r.name == name && r.status == Status::Unentschuldigt)
            .fold(0.0, |acc, r| acc + r.abwesenheitszeit);
        let record_absence_sum =
            RecordAbsenceSum::new(name, sum_entschuldigt as i32, sum_unentschuldigt as i32);
        records_with_absence_sum.push(record_absence_sum);
    }
    records_with_absence_sum
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}
