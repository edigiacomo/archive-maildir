use maildir::{MailEntry, Maildir};
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub enum MaildirArchiverError {
    IoError(std::io::Error),
    MaildirError(maildir::MaildirError),
}

impl From<std::io::Error> for MaildirArchiverError {
    fn from(value: std::io::Error) -> Self {
        MaildirArchiverError::IoError(value)
    }
}

impl From<maildir::MaildirError> for MaildirArchiverError {
    fn from(value: maildir::MaildirError) -> Self {
        MaildirArchiverError::MaildirError(value)
    }
}

pub trait MaildirArchiver {
    fn archive_email(
        &self,
        mail: &MailEntry,
        from_maildir: &Maildir,
        to_maildir: &Maildir,
    ) -> Result<(), MaildirArchiverError>;
}

struct DryRunMaildirArchiver {}

impl MaildirArchiver for DryRunMaildirArchiver {
    fn archive_email(
        &self,
        _mail: &MailEntry,
        _from_maildir: &Maildir,
        _to_maildir: &Maildir,
    ) -> Result<(), MaildirArchiverError> {
        Ok(())
    }
}

struct MoveMaildirArchiver {}

impl MaildirArchiver for MoveMaildirArchiver {
    fn archive_email(
        &self,
        mail: &MailEntry,
        from_maildir: &Maildir,
        to_maildir: &Maildir,
    ) -> Result<(), MaildirArchiverError> {
        let mut file = File::open(mail.path())?;
        let mut buff = Vec::<u8>::new();
        file.read_to_end(&mut buff)?;
        to_maildir.store_cur_with_flags(&buff, mail.flags())?;
        from_maildir.delete(mail.id())?;
        Ok(())
    }
}

struct CopyMaildirArchiver {}

impl MaildirArchiver for CopyMaildirArchiver {
    fn archive_email(
        &self,
        mail: &MailEntry,
        _from_maildir: &Maildir,
        to_maildir: &Maildir,
    ) -> Result<(), MaildirArchiverError> {
        let mut file = File::open(mail.path())?;
        let mut buff = Vec::<u8>::new();
        file.read_to_end(&mut buff)?;
        to_maildir.store_cur_with_flags(&buff, mail.flags())?;
        Ok(())
    }
}

pub enum ArchiveMode {
    Move,
    Copy,
    DryRun,
}

pub fn create_mail_archiver(mode: ArchiveMode) -> Box<dyn MaildirArchiver> {
    match mode {
        ArchiveMode::DryRun => Box::new(DryRunMaildirArchiver {}),
        ArchiveMode::Move => Box::new(MoveMaildirArchiver {}),
        ArchiveMode::Copy => Box::new(CopyMaildirArchiver {}),
    }
}
