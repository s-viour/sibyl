use anyhow::Result;
use chrono::{DateTime, Local};
use std::process::{Child, Command};

pub type SibylPID = u32;

/// bundles a command and a child, along with any other information that needs to be kept track-of
struct SibylProcess {
    pub cmdline: String,
    pub command: Command,
    pub child: Child,
    pub started: DateTime<Local>,
    pub pid: SibylPID,
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
        program: &String,
        args: &Vec<String>,
        mut command: Command,
    ) -> Result<SibylPID> {
        // build our own (owned) version of the command-line string
        // when Command::get_program and Command::get_args are stable, we won't have to do this
        let mut cmdline = program.clone();
        for arg in args {
            cmdline.push_str(&arg.clone());
        }

        let child = command.spawn()?;
        self.count += 1;
        let proc = SibylProcess {
            cmdline,
            command,
            child,
            started: Local::now(),
            pid: self.count,
        };
        self.processes.push(proc);

        Ok(self.count)
    }
}
