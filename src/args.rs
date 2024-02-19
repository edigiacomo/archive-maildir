use crate::archiver::*;
use clap::builder::PossibleValue;
use clap::{arg, command, ArgAction};
use log::LevelFilter;
use maildir::Maildir;
use std::path::PathBuf;
use time::macros::format_description;
use time::{Date, OffsetDateTime};

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
    now.replace_year(now.year() - 1).unwrap().date()
}

pub fn parse_args() -> ProgramOptions {
    let before_default = one_year_ago().to_string();
    let matches = command!()
        .version(env!("CARGO_PKG_VERSION"))
        .about("Archive emails from maildir, grouping them by date")
        .arg(
            arg!("prefix")
                .short('p')
                .long("prefix")
                .value_name("PREFIX")
                .help("Prefix format")
                .default_value(""),
        )
        .arg(
            arg!("suffix")
                .short('s')
                .long("suffix")
                .value_name("SUFFIX")
                .help("Suffix format")
                .default_value(""),
        )
        .arg(
            arg!("split-by")
                .short('S')
                .long("split-by")
                .value_name("PERIOD")
                .help("Set the split policy")
                .value_parser([
                    PossibleValue::new("year"),
                    PossibleValue::new("month"),
                    PossibleValue::new("day"),
                    PossibleValue::new("none"),
                ])
                .default_value("year"),
        )
        .arg(
            arg!("mode")
                .short('m')
                .long("mode")
                .help("Archive mode")
                .value_parser([
                    PossibleValue::new("copy"),
                    PossibleValue::new("move"),
                    PossibleValue::new("dry-run"),
                ])
                .default_value("dry-run"),
        )
        .arg(
            arg!("before")
                .short('b')
                .long("before")
                .default_value(before_default)
                .value_name("YYYY-mm-dd")
                .help("Archive emails before the given date"),
        )
        .arg(
            arg!("verbose")
                .short('v')
                .long("verbose")
                .help("Set verbosity")
                .action(ArgAction::Count),
        )
        .arg(
            arg!("input-maildir")
                .required(true)
                .value_name("INPUT_PATH")
                .help("Input maildir path")
                .index(1),
        )
        .arg(
            arg!("output-dir")
                .required(true)
                .value_name("OUTPUT_PATH")
                .help("Output directory for archive maildirs")
                .index(2),
        )
        .get_matches();
    let dateformat = format_description!("[year]-[month]-[day]");
    let p = ProgramOptions {
        input_maildir: (*matches.get_one::<String>("input-maildir").unwrap().clone()).into(),
        output_dir: (*matches.get_one::<PathBuf>("output-dir").unwrap().clone()).to_path_buf(),
        before: Date::parse(matches.get_one::<String>("before").unwrap(), &dateformat).unwrap(),
        prefix: matches.get_one::<String>("prefix").unwrap().clone(),
        suffix: matches.get_one::<String>("suffix").unwrap().clone(),
        split_by: match matches.get_one::<String>("split-by").unwrap().as_str() {
            "day" => SplitBy::Day,
            "month" => SplitBy::Month,
            "year" => SplitBy::Year,
            _ => SplitBy::None,
        },
        verbosity: match matches.get_count("verbose") {
            0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            _ => LevelFilter::Debug,
        },
        archive_mode: match matches.get_one::<String>("mode").unwrap().as_str() {
            "copy" => ArchiveMode::Copy,
            "move" => ArchiveMode::Move,
            _ => ArchiveMode::DryRun,
        },
    };
    p
}
