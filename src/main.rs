use archive_maildir::archiver::*;

use maildir::Maildir;
use chrono::Datelike;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use clap::{App, Arg};
use log::{debug, error, info, LevelFilter};
use simple_logger::SimpleLogger;
use std::path::PathBuf;

enum SplitBy {
    Year,
    Day,
    Month,
}

struct ProgramOptions {
    input_path: Maildir,
    before: Option<String>,
    output_dir: PathBuf,
    archive_mode: ArchiveMode,
    prefix: String,
    suffix: String,
    split_by: SplitBy,
    verbosity: LevelFilter,
}

fn parse_args() -> ProgramOptions {
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

fn main() {
    let opts = parse_args();
    SimpleLogger::new()
        .with_level(opts.verbosity)
        .init()
        .unwrap();
    let before = match opts.before {
        // NOTE: the value is already validated
        Some(s) => NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap_or_else(|e| {
            error!("While parsing time threshold: {}", e);
            std::process::exit(1);
        }),
        None => {
            let now = Utc::now().naive_utc().date();
            now.clone().with_year(now.year() - 1).unwrap_or_else(|| {
                error!("While processing time threshold");
                std::process::exit(1);
            })
        }
    };
    let mail_archiver = create_mail_archiver(opts.archive_mode);
    info!(
        "Archiving emails in maildir {} older than {}",
        opts.input_path.path().display(),
        before
    );
    for mailentry in opts.input_path.list_cur() {
        match mailentry {
            Ok(mut mail) => {
                match mail.received() {
                    Ok(timestamp) => {
                        let date = NaiveDateTime::from_timestamp(timestamp, 0).date();
                        if date < before {
                            debug!(
                                "Email {} date {} is older than threshold {}",
                                mail.id(),
                                date,
                                before
                            );
                            let mut output_folder = PathBuf::from(&opts.output_dir);
                            output_folder.push(format!(
                                "{}{}{}",
                                opts.prefix,
                                date.format(match opts.split_by {
                                    SplitBy::Year => "%Y",
                                    SplitBy::Month => "%Y-%m",
                                    SplitBy::Day => "%Y-%m-%d",
                                }),
                                opts.suffix
                            ));
                            let to_maildir = Maildir::from(output_folder);
                            to_maildir.create_dirs().unwrap();
                            info!(
                                "Archiving email {} from folder {} to folder {}",
                                mail.id(),
                                opts.input_path.path().display(),
                                to_maildir.path().display()
                            );
                            mail_archiver
                                .archive_email(&mail, &opts.input_path, &to_maildir)
                                .unwrap();
                        } else {
                            debug!(
                                "Ignoring email {}: date {} is older than threshold {}",
                                mail.id(),
                                date,
                                before
                            );
                        }
                    }
                    Err(e) => error!(
                        "Error while extracting date from email {}: {}",
                        mail.id(),
                        e
                    ),
                };
            }
            Err(e) => error!("{}", e),
        }
    }
}
