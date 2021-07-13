use crate::logging::{LogHandler, LogName};
use crate::processing::ProcessHandler;
use crate::{Request, Response};
use anyhow::{Context, Result};
use chrono::{Local, Utc};
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::ffi::{OsStr, OsString};
use std::fs::{metadata, read_dir, OpenOptions};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::SystemTime;
use typetag;

/// structure containing all resources that commands may need to access
pub struct CommandContext {
    pub loghandler: LogHandler,
    pub prochandler: ProcessHandler,
}

/// trait that represents an action executable by the server
///
/// all command-structures implement this trait
#[typetag::serde(tag = "type")]
pub trait Action {
    fn execute(&self, req: &Request, ctx: &mut CommandContext) -> Result<Response>;
}

/// command-structure for the `once` command
///
/// action that describes a program to be run once
/// with output logged and stored in a temporary file
#[derive(Serialize, Deserialize, Debug)]
pub struct CmdOnce {
    pub program: OsString,
    pub args: Vec<OsString>,
}

// implement the ability to create a CmdOnce from clap's ArgMatches
// consider moving this to a utility file or in sibyl.rs
impl From<&ArgMatches<'_>> for CmdOnce {
    fn from(matches: &ArgMatches) -> Self {
        let cmdline = matches.values_of("cmd").unwrap().collect::<Vec<_>>();

        let (program, args) = cmdline.split_at(1);
        let program: OsString = OsString::from(program[0]);
        let args: Vec<OsString> = args.iter().map(OsString::from).collect();

        CmdOnce { program, args }
    }
}

// implement LogName for Once since it requires the ability to create logfiles
impl LogName for CmdOnce {
    fn log_name(&self) -> PathBuf {
        let mut v: Vec<&OsStr> = vec![&self.program];
        let mut args: Vec<&OsStr> = self.args.iter().map(|s| s.as_os_str()).collect();
        v.append(&mut args);

        let mut filename = OsString::new();
        if v.len() == 1 {
            filename.push(v[0]);
        } else {
            for arg in v.iter().take(v.len() - 2) {
                filename.push(arg);
                filename.push("_");
            }
            filename.push(v[args.len() - 1]);
        }
        let timestamp = format!("_{}", Local::now());
        let timestamp: String = timestamp
            .chars()
            .map(|c| match c {
                ' ' => '-',
                _ => c,
            })
            .collect();
        filename.push(timestamp);

        PathBuf::from(filename)
    }
}

#[typetag::serde]
impl Action for CmdOnce {
    fn execute(&self, _req: &Request, ctx: &mut CommandContext) -> Result<Response> {
        let logfile = ctx.loghandler.create_log(self)?;
        let output_file = logfile.open()?;

        let mut cmd = Command::new(&self.program);
        cmd.args(&self.args)
            .stdout(Stdio::from(output_file))
            .stderr(Stdio::null());

        let pid = ctx
            .prochandler
            .create_process(&self.program, &self.args, logfile.get_path(), cmd)
            .context("failed to create process!")?;

        Ok(Response {
            msg: format!(
                "successfully executed process: {} | sibyl pid: {}",
                &self.program.to_str().unwrap(),
                pid
            ),
        })
    }
}

/// command-structure for the `latest` command
///
/// retrieves and sends the latest log file to the client
#[derive(Serialize, Deserialize)]
pub struct CmdLatest;

#[typetag::serde]
impl Action for CmdLatest {
    fn execute(&self, _req: &Request, ctx: &mut CommandContext) -> Result<Response> {
        let path = ctx.loghandler.log_directory();
        let mut latest_file = PathBuf::new();
        let mut last_modified = SystemTime::UNIX_EPOCH;

        for file in read_dir(path)? {
            let file = file?;
            let ftime = metadata(file.path())?.modified()?;
            if ftime > last_modified {
                last_modified = ftime;
                latest_file = file.path();
            }
        }

        let mut file = OpenOptions::new()
            .read(true)
            .write(false)
            .open(latest_file)?;

        let mut s = String::new();
        file.read_to_string(&mut s)?;

        Ok(Response { msg: s })
    }
}

/// command-structure for the `ping` command
///
/// for now, just responds with 'pong!'
// we should change this to actually time the ping
#[derive(Serialize, Deserialize)]
pub struct CmdPing;

#[typetag::serde]
impl Action for CmdPing {
    fn execute(&self, req: &Request, _ctx: &mut CommandContext) -> Result<Response> {
        let now = Utc::now();
        let pingtime = now - req.time;

        Ok(Response {
            msg: format!("pong! {}ms", pingtime.num_milliseconds()),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct CmdStatus {
    pub pid: u32,
}

impl From<&ArgMatches<'_>> for CmdStatus {
    fn from(matches: &ArgMatches) -> Self {
        let pid: u32 = matches
            .value_of("pid")
            .unwrap()
            .parse()
            .expect("failed to parse pid as integer!");
        CmdStatus { pid }
    }
}

#[typetag::serde]
impl Action for CmdStatus {
    fn execute(&self, _req: &Request, ctx: &mut CommandContext) -> Result<Response> {
        Ok(match ctx.prochandler.get_process_status(self.pid) {
            Some(status) => Response {
                msg: format!("{}", status),
            },
            None => Response {
                msg: format!("no process found with pid {}", self.pid),
            },
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct CmdList;

#[typetag::serde]
impl Action for CmdList {
    fn execute(&self, _req: &Request, ctx: &mut CommandContext) -> Result<Response> {
        let mut msg = format!("list of processes:\n");
        
        for proc in ctx.prochandler.all_processes() {
            writeln!(&mut msg, "  SPID: {} - {}", proc.pid, proc.cmdline.to_str().unwrap())?;
        }

        Ok(Response {
            msg,
        })
    }
}