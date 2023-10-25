use time::macros::format_description;
use time::Duration;
use time::Time;
//tutorial-read-01.rs
use std::{env, error::Error, ffi::OsString, fs::File, process};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Record {
    abwesenheitszeit: String,
    name: String,
    status: String,
    #[serde(rename = "Aktualisiert am")]
    aktualisiert: String,
}

fn run() -> Result<(), Box<dyn Error>> {
    let abwesenheit_time_format = format_description!("[hour]:[minute]");
    let abwesenheit_date_format = format_description!("[day]");

    let file_path = get_first_arg()?;
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new().delimiter(b';').from_reader(file);
    for result in rdr.deserialize() {
        let mut record: Record = result?;
        if record.abwesenheitszeit.contains(" - ") {
            if record.abwesenheitszeit.ends_with(")") {
                let start_zeit =
                    Time::parse(&record.abwesenheitszeit[12..17], &abwesenheit_time_format)
                        .unwrap();
                let end_zeit =
                    Time::parse(&record.abwesenheitszeit[20..25], &abwesenheit_time_format)
                        .unwrap();
                let duration: Duration = end_zeit - start_zeit;
                record.abwesenheitszeit = duration.whole_minutes().to_string();
            } else {
                println!("{:?}", &record.abwesenheitszeit[0..10]);
                let start_datum = Time::parse("01", &abwesenheit_date_format).unwrap();
                let end_datum = Time::parse("15", &abwesenheit_date_format).unwrap();
                let duration: Duration = end_datum - start_datum;
                record.abwesenheitszeit = duration.whole_days().to_string();
            }
        } else {
            record.abwesenheitszeit = String::from("268");
        }
        println!("{:?}", record);
    }
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

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}
