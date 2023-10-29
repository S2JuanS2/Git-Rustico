use std::{fs::{File, OpenOptions}, io::{self, Write}};

use crate::errors::GitError;

pub enum LogFile {
    FileResult(File),
    StderrResult(io::Stderr),
}

impl LogFile {
    pub fn new(path: &str) -> Result<LogFile, GitError> {
        match open_log_file(path)
        {
            Ok(file) => Ok(LogFile::FileResult(file)),
            Err(_) => Ok(LogFile::StderrResult(io::stderr())),
        }
    }

    pub fn sync_all(&mut self) -> Result<(), GitError> {
        match self {
            LogFile::FileResult(file) => {
                if file.sync_all().is_err()
                {
                    return Err(GitError::LogfileSyncError);
                };
                Ok(())
            },
            LogFile::StderrResult(_) => Ok(()),
        }
    }
}

fn open_log_file(log_path: &str) -> Result<File, GitError> {
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        {
            Ok(file) => Ok(file),
            Err(_) => return Err(GitError::LogfileOpenError),
        }
}

impl Write for LogFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            LogFile::FileResult(file) => file.write(buf),
            LogFile::StderrResult(stderr) => stderr.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            LogFile::FileResult(file) => file.flush(),
            LogFile::StderrResult(stderr) => stderr.flush(),
        }
    }
    
}