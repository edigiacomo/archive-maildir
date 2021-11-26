use archive_maildir::archiver::*;
use archive_maildir::args::*;

use chrono::Datelike;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use log::{debug, error, info};
use maildir::Maildir;
use simple_logger::SimpleLogger;
use std::path::PathBuf;

fn main() {
    let opts = parse_args();
    SimpleLogger::new()
        .with_level(opts.verbosity)
        .init()
        .unwrap();
    let before = match opts.before {
        // NOTE: the value is already validated
        Some(ref s) => NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap_or_else(|e| {
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
