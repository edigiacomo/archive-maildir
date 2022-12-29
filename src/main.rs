use archive_maildir::archiver::*;
use archive_maildir::args::*;

use time::OffsetDateTime;
use time::macros::format_description;
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
        "Archiving emails older than {}",
        opts.input_maildir.path().display(),
    );
    let maildir_size = opts.input_maildir.count_cur();
    let archived = opts
        .input_maildir
        .list_cur()
        .enumerate()
        .filter_map(|(index, entry)| match entry {
            Ok(m) => {
                debug!("{}/{} email {}", index + 1, maildir_size, m.id());
                Some(m)
            }
            Err(e) => {
                error!("{}", e);
                None
            }
        })
        .filter_map(|mut mail| match mail.received() {
            Ok(timestamp) => OffsetDateTime::from_unix_timestamp(timestamp).ok().map(|dt| (mail, dt)),
            Err(e) => {
                error!("{}", e);
                None
            }
        })
        .filter(|(mail, maildate)| {
            if maildate.date() < opts.before {
                debug!(
                    "Email {} with timestamp {} is older than threshold",
                    mail.id(),
                    maildate
                );
                true
            } else {
                debug!(
                    "Email {} with timestamp {} is newer than threshold",
                    mail.id(),
                    maildate
                );
                false
            }
        })
        .filter_map(|(mail, maildate)| {
            let mut output_folder = PathBuf::from(&opts.output_dir);
            let dateformat = match opts.split_by {
                SplitBy::Year => format_description!("[year]"),
                SplitBy::Month => format_description!("[year]-[month]"),
                SplitBy::Day => format_description!("[year]-[month]-[day]"),
                SplitBy::None => format_description!(""),
            };
            output_folder.push(format!(
                "{}{}{}",
                opts.prefix,
                maildate.format(&dateformat).unwrap(),
                opts.suffix
            ));
            let to_maildir = Maildir::from(output_folder);
            match mail_archiver.archive_email(&mail, &opts.input_maildir, &to_maildir) {
                Err(e) => {
                    error!(
                        "Error while archiving email {} from folder {} to folder {}: {}",
                        mail.id(),
                        opts.input_maildir.path().display(),
                        to_maildir.path().display(),
                        e
                    );
                    None
                }
                Ok(()) => {
                    info!(
                        "Email {} from folder {} archived to folder {}",
                        mail.id(),
                        opts.input_maildir.path().display(),
                        to_maildir.path().display()
                    );
                    Some((mail.id().to_string(), to_maildir))
                }
            }
        });
    info!("Archived {}/{} email", archived.count(), maildir_size);
}
