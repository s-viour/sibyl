use anyhow::Result;
use chrono::{DateTime, Local};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

pub type SibylPID = u32;

/// bundles a command and a child, along with any other information that needs to be kept track-of
pub struct SibylProcess {
    pub cmdline: OsString,
    pub command: Command,
    pub child: Child,
    pub started: DateTime<Local>,
    pub pid: SibylPID,
    pub log_file: PathBuf,
}

pub enum ProcessWaitStatus {
    Running(u32),
    Exited(Option<i32>),
    Unknown,
}

impl fmt::Display for ProcessWaitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            ProcessWaitStatus::Running(p) => write!(f, "running (pid {})", p),
            ProcessWaitStatus::Exited(Some(p)) => write!(f, "exited (exit code {})", p),
            ProcessWaitStatus::Exited(None) => write!(f, "exited (no exit code)"),
            ProcessWaitStatus::Unknown => write!(f, "unknown"),
        }
    }
}

pub struct ProcessStatus {
    pub cmdline: OsString,
    pub started: DateTime<Local>,
    pub internal_pid: SibylPID,
    pub os_pid: u32,
    pub status: ProcessWaitStatus,
    pub log_path: PathBuf,
}

impl fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "process status for ({})", self.internal_pid)?;
        writeln!(f, "  command line : {}", self.cmdline.to_str().unwrap())?;
        writeln!(f, "  started at   : {}", self.started)?;
        writeln!(f, "  OS PID       : {}", self.os_pid)?;
        writeln!(f, "  wait status  : {}", self.status)?;
        writeln!(f, "  log file     : {}", self.log_path.display())
    }
}

/// state structure for the process handler
/// the process handler has functions that are invoked by commands to interact with processes
pub struct ProcessHandler {
    count: SibylPID,
    processes: Vec<SibylProcess>,
}

impl ProcessHandler {
    /// creates a new process handler
    pub fn new() -> ProcessHandler {
        ProcessHandler {
            count: 0,
            processes: Vec::new(),
        }
    }

    /// create a process running under the process handler
    /// # Arguments
    /// * `cmd` - a Command structure to spawn from
    pub fn create_process(
        &mut self,
        program: &OsStr,
        args: &[OsString],
        log_path: &Path,
        mut command: Command,
    ) -> Result<SibylPID> {
        // build our own (owned) version of the command-line string
        // when Command::get_program and Command::get_args are stable, we won't have to do this
        let mut cmdline = OsString::from(program);
        for arg in args {
            cmdline.push(&arg.clone());
        }

        let child = command.spawn()?;
        self.count += 1;
        let proc = SibylProcess {
            cmdline,
            command,
            child,
            started: Local::now(),
            pid: self.count,
            log_file: PathBuf::from(log_path),
        };
        self.processes.push(proc);

        Ok(self.count)
    }

    pub fn get_process_by_pid(&self, pid: SibylPID) -> Option<&SibylProcess> {
        self.processes.iter().find(|&proc| proc.pid == pid)
    }

    pub fn get_process_status(&mut self, pid: SibylPID) -> Option<ProcessStatus> {
        let proc = self.processes.iter_mut().find(|proc| proc.pid == pid);
        if let Some(proc) = proc {
            let cmdline = proc.cmdline.clone();
            let started = proc.started;
            let internal_pid = pid;
            let os_pid = proc.child.id();
            let status = match proc.child.try_wait() {
                Ok(Some(status)) => ProcessWaitStatus::Exited(status.code()),
                Ok(None) => ProcessWaitStatus::Running(os_pid),
                Err(_) => ProcessWaitStatus::Unknown,
            };
            let log_path = proc.log_file.clone();

            Some(ProcessStatus {
                cmdline,
                started,
                internal_pid,
                os_pid,
                status,
                log_path,
            })
        } else {
            None
        }
    }

    pub fn all_processes(&self) -> &[SibylProcess] {
        &self.processes.as_slice()
    }
}

impl Default for ProcessHandler {
    fn default() -> Self {
        Self::new()
    }
}
