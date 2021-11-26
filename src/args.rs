use crate::archiver::*;
use chrono::NaiveDate;
use clap::{App, Arg};
use log::LevelFilter;
use maildir::Maildir;
use std::path::PathBuf;

pub struct ProgramOptions {
    pub input_path: Maildir,
    pub before: Option<String>,
    pub output_dir: PathBuf,
    pub archive_mode: ArchiveMode,
    pub prefix: String,
    pub suffix: String,
    pub split_by: SplitBy,
    pub verbosity: LevelFilter,
}

pub enum SplitBy {
    Year,
    Day,
    Month,
}

pub fn parse_args() -> ProgramOptions {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Archive emails from maildir")
        .arg(
            Arg::with_name("output-dir")
                .short("o")
                .long("output-dir")
                .value_name("PATH")
                .help("Output directory")
                .takes_value(true)
                .default_value("."),
        )
        .arg(
            Arg::with_name("prefix")
                .short("p")
                .long("prefix")
                .value_name("PREFIX")
                .help("Prefix format")
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::with_name("suffix")
                .short("s")
                .long("suffix")
                .value_name("SUFFIX")
                .help("Suffix format")
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::with_name("split-by")
                .short("S")
                .long("split-by")
                .value_name("PERIOD")
                .help("Split by")
                .takes_value(true)
                .possible_value("year")
                .possible_value("month")
                .possible_value("day")
                .default_value("year"),
        )
        .arg(
            Arg::with_name("mode")
                .short("m")
                .long("mode")
                .help("Archive mode")
                .possible_value("copy")
                .possible_value("move")
                .possible_value("dry-run")
                .default_value("dry-run"),
        )
        .arg(
            Arg::with_name("before")
                .short("b")
                .long("before")
                .value_name("YYYY-mm-dd")
                .validator(|v| match NaiveDate::parse_from_str(&v, "%Y-%m-%d") {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("{}", e)),
                })
                .help("Archive emails before the given date"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Set verbosity")
                .multiple(true)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("PATH")
                .required(true)
                .help("Maildir path")
                .index(1),
        )
        .get_matches();
    let p = ProgramOptions {
        input_path: matches.value_of("PATH").unwrap().into(),
        output_dir: matches.value_of("output-dir").unwrap().into(),
        before: matches.value_of("before").map(|s| s.to_string()).or(None),
        prefix: matches.value_of("prefix").unwrap().into(),
        suffix: matches.value_of("suffix").unwrap().into(),
        split_by: match matches.value_of("split-by").unwrap() {
            "day" => SplitBy::Day,
            "month" => SplitBy::Month,
            _ => SplitBy::Year,
        },
        verbosity: match matches.occurrences_of("verbose") {
            0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            _ => LevelFilter::Debug,
        },
        archive_mode: match matches.value_of("mode").unwrap() {
            "copy" => ArchiveMode::Copy,
            "move" => ArchiveMode::Move,
            _ => ArchiveMode::DryRun,
        },
    };
    p
}
