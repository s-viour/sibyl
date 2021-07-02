use anyhow::Result;
use log::debug;
use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::path::{Path, PathBuf};

/// trait that describes any command
/// that requires the loghandler to be able
/// to create a log file for it
pub trait LogName {
    /// trait function that returns a PathBuf to where the log should be created
    /// any command-structure that needs to create logfiles should implement this trait
    fn log_name(&self) -> PathBuf;
}

/// structure that represents a logfile
/// currently only stores a path, but might be expanded later
/// to include formatted writing utilities
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

/// state structure for the log handler
/// the log handler has methods that are invoked by commands
/// to generate, access, and manage log files generated by processes
pub struct LogHandler {
    directory: PathBuf,
    logs: HashMap<PathBuf, LogFile>,
}

impl LogHandler {
    /// create a log handler
    /// # Arguments
    /// * `p` - root directory for the log handler to work in
    pub fn new(p: &Path) -> Self {
        let mut directory = PathBuf::new();
        directory.push(p);

        Self {
            directory,
            logs: HashMap::new(),
        }
    }

    /// returns the root path of the log handler
    pub fn log_directory(&self) -> &Path {
        &self.directory.as_path()
    }

    /// returns a LogFile structure that the callee can use
    /// to access the log file that was created
    /// # Arguments
    /// * `name` - a reference to a command-struct implementing LogName
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
