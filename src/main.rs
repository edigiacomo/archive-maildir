use archive_maildir::archiver::*;
use archive_maildir::args::*;

use chrono::NaiveDateTime;
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
    let mail_archiver = create_mail_archiver(opts.archive_mode);
    info!(
        "Archiving emails in maildir {} older than {}",
        opts.input_maildir.path().display(),
        opts.before
    );
    for mailentry in opts.input_maildir.list_cur() {
        match mailentry {
            Ok(mut mail) => {
                match mail.received() {
                    Ok(timestamp) => {
                        let date = NaiveDateTime::from_timestamp(timestamp, 0).date();
                        if date < opts.before {
                            debug!(
                                "Email {} date {} is older than threshold {}",
                                mail.id(),
                                date,
                                opts.before
                            );
                            let mut output_folder = PathBuf::from(&opts.output_dir);
                            output_folder.push(format!(
                                    "{}{}{}",
                                    opts.prefix,
                                    date.format(match opts.split_by {
                                        SplitBy::Year => "%Y",
                                        SplitBy::Month => "%Y-%m",
                                        SplitBy::Day => "%Y-%m-%d",
                                        SplitBy::None => "",
                                    }),
                                    opts.suffix
                            ));
                            let to_maildir = Maildir::from(output_folder);
                            to_maildir.create_dirs().unwrap();
                            info!(
                                "Archiving email {} from folder {} to folder {}",
                                mail.id(),
                                opts.input_maildir.path().display(),
                                to_maildir.path().display()
                            );
                            mail_archiver
                                .archive_email(&mail, &opts.input_maildir, &to_maildir)
                                .unwrap();
                            } else {
                                debug!(
                                    "Ignoring email {}: date {} is older than threshold {}",
                                    mail.id(),
                                    date,
                                    opts.before
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
