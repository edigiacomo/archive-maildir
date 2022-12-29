use crate::archiver::*;
use time::{Date, OffsetDateTime};
use time::macros::format_description;
use clap::{value_t_or_exit, Arg, Command};
use log::LevelFilter;
use maildir::Maildir;
use std::path::PathBuf;

pub struct ProgramOptions {
    pub input_maildir: Maildir,
    pub before: Date,
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

fn one_year_ago() -> Date {
    let now = OffsetDateTime::now_utc();
    now.clone().replace_year(now.year() - 1).unwrap().date()
}

pub fn parse_args() -> ProgramOptions {
    let before_default = one_year_ago().to_string();
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Archive emails from maildir, grouping them by date")
        .arg(
            Arg::new("prefix")
                .short('p')
                .long("prefix")
                .value_name("PREFIX")
                .help("Prefix format")
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::new("suffix")
                .short('s')
                .long("suffix")
                .value_name("SUFFIX")
                .help("Suffix format")
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::new("split-by")
                .short('S')
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
            Arg::new("mode")
                .short('m')
                .long("mode")
                .help("Archive mode")
                .possible_value("copy")
                .possible_value("move")
                .possible_value("dry-run")
                .default_value("dry-run"),
        )
        .arg(
            Arg::new("before")
                .short('b')
                .long("before")
                .default_value(&before_default)
                .value_name("YYYY-mm-dd")
                .help("Archive emails before the given date"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Set verbosity")
                .multiple(true)
                .takes_value(false),
        )
        .arg(
            Arg::new("input-maildir")
                .required(true)
                .value_name("INPUT_PATH")
                .help("Input maildir path")
                .index(1),
        )
        .arg(
            Arg::new("output-dir")
                .required(true)
                .value_name("OUTPUT_PATH")
                .help("Output directory for archive maildirs")
                .index(2),
        )
        .get_matches();
    let dateformat = format_description!("[year]-[month]-[day]");
    let p = ProgramOptions {
        // Unfortunately, Maildir doesn't implement trait FromStr
        input_maildir: value_t_or_exit!(matches, "input-maildir", String).into(),
        output_dir: value_t_or_exit!(matches, "output-dir", PathBuf),
        before: Date::parse(&value_t_or_exit!(matches, "before", String), &dateformat).unwrap(),
        prefix: value_t_or_exit!(matches, "prefix", String),
        suffix: value_t_or_exit!(matches, "suffix", String),
        split_by: match matches.value_of("split-by").unwrap() {
            "day" => SplitBy::Day,
            "month" => SplitBy::Month,
            "year" => SplitBy::Year,
            _ => SplitBy::None,
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
