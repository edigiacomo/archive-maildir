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
    opts.input_maildir.list_cur()
        .filter_map(|entry| match entry {
            Ok(m) => Some(m),
            Err(e) => {
                error!("{}", e);
                None
            }
        })
    .filter_map(|mut mail| match mail.received() {
        Ok(timestamp) => Some((mail, NaiveDateTime::from_timestamp(timestamp))),
        Err(e) => {
            error!("{}", e);
            None
        }
    })
    .filter(|(mail, maildate)| {
        if maildate.date() < opts.before {
            debug!("Email {} with timestamp {} is older than threshold", mail.id(), maildate);
            true
        } else {
            debug!("Email {} with timestamp {} is newer than threshold", mail.id(), maildate);
            false
        }
    })
    .for_each(|(mail, maildate)| {
        let mut output_folder = PathBuf::from(&opts.output_dir);
        output_folder.push(format!(
                "{}{}{}",
                opts.prefix,
                maildate.format(match opts.split_by {
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
        match mail_archiver.archive_email(&mail, &opts.input_maildir, &to_maildir) {
            Err(e) => error!("{}", e),
            _ => {},
        }
    });
}
