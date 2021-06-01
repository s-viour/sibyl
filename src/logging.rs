use std::collections::HashMap;
use std::fs::{File, OpenOptions, create_dir_all};
use std::path::{Path, PathBuf};
use anyhow::Result;
use log::{debug};

// trait that describes any command
// that requires the loghandler to be able
// to create a log file for it
pub trait LogName {
    fn log_name(&self) -> PathBuf;
}

pub struct LogFile {
    path: PathBuf,
}

impl LogFile {
    pub fn open(&self) -> Result<File> {
        let f = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&self.path)?;

        Ok(f)
    }
}

pub struct LogHandler {
    directory: PathBuf,
    logs: HashMap<PathBuf, LogFile>,
}

impl LogHandler {
    pub fn new(p: &Path) -> Self {
        let mut directory = PathBuf::new();
        directory.push(p);

        Self {
            directory,
            logs: HashMap::new(),
        }
    }

    pub fn log_directory(&self) -> &Path {
        &self.directory.as_path()
    }

    pub fn create_log(&mut self, name: &impl LogName) -> Result<&LogFile> {
        let mut path = PathBuf::new();
        path.push(&self.directory);
        debug!("log file directory does not exist. creating...");
        create_dir_all(&path)?;
        let log_name = name.log_name();
        path.push(&log_name);
        path.set_extension("slog");
        debug!("creating output log at {:?}", path);

        self.logs.insert(log_name.clone(), LogFile { path });
        Ok(self.logs.get(&log_name).unwrap())
    }
}