use maildir::{MailEntry, Maildir};
use std::fmt;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub enum MaildirArchiverError {
    IoError(std::io::Error),
    MaildirError(maildir::MaildirError),
}

impl fmt::Display for MaildirArchiverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            MaildirArchiverError::IoError(e) => format!("{}", e),
            MaildirArchiverError::MaildirError(e) => format!("{}", e),
        };
        write!(f, "{}", msg)
    }
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

/// Trait implemented by the mail archiver.
///
/// The function [`MaildirArchiver::archive_email`] is generally used in a loop.
pub trait MaildirArchiver {
    fn archive_email(
        &self,
        mail: &MailEntry,
        from_maildir: &Maildir,
        to_maildir: &Maildir,
    ) -> Result<(), MaildirArchiverError>;
}

/// Dry run archiver
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

/// Archiver that move email from one maildir to another
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

        to_maildir.create_dirs()?;
        file.read_to_end(&mut buff)?;
        to_maildir.store_cur_with_flags(&buff, mail.flags())?;
        from_maildir.delete(mail.id())?;
        Ok(())
    }
}

/// Archiver that copy email from one maildir to another
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

        to_maildir.create_dirs()?;
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

/// Factory method that creates an archiver
pub fn create_mail_archiver(mode: ArchiveMode) -> Box<dyn MaildirArchiver> {
    match mode {
        ArchiveMode::DryRun => Box::new(DryRunMaildirArchiver {}),
        ArchiveMode::Move => Box::new(MoveMaildirArchiver {}),
        ArchiveMode::Copy => Box::new(CopyMaildirArchiver {}),
    }
}

#[cfg(test)]
mod tests {
    use maildir::Maildir;
    use std::path::PathBuf;

    struct MaildirRaii {
        basedir: PathBuf,
        input_maildir: Maildir,
        output_maildir: Maildir,
    }

    impl MaildirRaii {
        fn new() -> Self {
            use mktemp::Temp;
            let basedir = Temp::new_dir().unwrap().to_path_buf();
            println!("{}", basedir.display());
            let input_maildir = Maildir::from(basedir.join("in"));
            let output_maildir = Maildir::from(basedir.join("out"));
            input_maildir.create_dirs().unwrap();

            let filename = "1463868505.38518452d49213cb409aa1db32f53184:2,S";
            std::fs::copy(
                format!("testdata/maildir1/cur/{}", filename),
                input_maildir.path().join("cur").join(filename),
            )
            .unwrap();

            MaildirRaii {
                basedir: basedir,
                input_maildir: input_maildir,
                output_maildir: output_maildir,
            }
        }
    }

    impl Drop for MaildirRaii {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.basedir).unwrap();
        }
    }

    #[test]
    fn test_move_archive_email() {
        use crate::archiver::MaildirArchiver;
        use crate::archiver::MoveMaildirArchiver;

        let maildir = MaildirRaii::new();
        let archiver = MoveMaildirArchiver {};
        let mail = maildir.input_maildir.list_cur().next().unwrap().unwrap();

        assert_eq!(maildir.input_maildir.count_cur(), 1);
        assert_eq!(maildir.output_maildir.count_cur(), 0);
        archiver
            .archive_email(&mail, &maildir.input_maildir, &maildir.output_maildir)
            .unwrap();
        assert_eq!(maildir.input_maildir.count_cur(), 0);
        assert!(maildir.output_maildir.path().exists());
        assert_eq!(maildir.output_maildir.count_cur(), 1);
    }

    #[test]
    fn test_copy_archive_email() {
        use crate::archiver::CopyMaildirArchiver;
        use crate::archiver::MaildirArchiver;

        let maildir = MaildirRaii::new();
        let archiver = CopyMaildirArchiver {};
        let mail = maildir.input_maildir.list_cur().next().unwrap().unwrap();

        assert_eq!(maildir.input_maildir.count_cur(), 1);
        assert_eq!(maildir.output_maildir.count_cur(), 0);
        archiver
            .archive_email(&mail, &maildir.input_maildir, &maildir.output_maildir)
            .unwrap();
        assert_eq!(maildir.input_maildir.count_cur(), 1);
        assert!(maildir.output_maildir.path().exists());
        assert_eq!(maildir.output_maildir.count_cur(), 1);
    }

    #[test]
    fn test_dryrun_archive_email() {
        use crate::archiver::DryRunMaildirArchiver;
        use crate::archiver::MaildirArchiver;

        let maildir = MaildirRaii::new();
        let archiver = DryRunMaildirArchiver {};
        let mail = maildir.input_maildir.list_cur().next().unwrap().unwrap();

        assert_eq!(maildir.input_maildir.count_cur(), 1);
        assert_eq!(maildir.output_maildir.count_cur(), 0);
        archiver
            .archive_email(&mail, &maildir.input_maildir, &maildir.output_maildir)
            .unwrap();
        assert_eq!(maildir.input_maildir.count_cur(), 1);
        assert!(!maildir.output_maildir.path().exists());
    }
}
