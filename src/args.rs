use crate::archiver::*;
use chrono::{Datelike, NaiveDate, Utc};
use clap::{value_t_or_exit, App, Arg};
use log::LevelFilter;
use maildir::Maildir;
use std::path::PathBuf;

pub struct ProgramOptions {
    pub input_maildir: Maildir,
    pub before: NaiveDate,
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
    None,
}

fn one_year_ago() -> NaiveDate {
    let now = Utc::now().naive_utc().date();
    now.clone().with_year(now.year() - 1).unwrap()
}

pub fn parse_args() -> ProgramOptions {
    let before_default = &(one_year_ago().to_string());
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Archive emails from maildir, grouping them by date")
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
                .help("Set the split policy")
                .takes_value(true)
                .possible_value("year")
                .possible_value("month")
                .possible_value("day")
                .possible_value("none")
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
                .default_value(&before_default)
                .value_name("YYYY-mm-dd")
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
            Arg::with_name("input-maildir")
                .required(true)
                .value_name("INPUT_PATH")
                .help("Input maildir path")
                .index(1),
        )
        .arg(
            Arg::with_name("output-dir")
                .required(true)
                .value_name("OUTPUT_PATH")
                .help("Output directory for archive maildirs")
                .index(2)
        )
        .get_matches();
    let p = ProgramOptions {
        input_maildir: matches.value_of("input-maildir").unwrap().into(),
        output_dir: matches.value_of("output-dir").unwrap().into(),
        before: value_t_or_exit!(matches, "before", NaiveDate),
        prefix: matches.value_of("prefix").unwrap().into(),
        suffix: matches.value_of("suffix").unwrap().into(),
        split_by: match matches.value_of("split-by").unwrap() {
            "day" => SplitBy::Day,
            "month" => SplitBy::Month,
            "year" => SplitBy::Year,
            _  => SplitBy::None,
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
