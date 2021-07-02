use crate::logging::{LogHandler, LogName};
use crate::{Request, Response};
use anyhow::{Context, Result};
use chrono::Utc;
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::fs::{metadata, read_dir, OpenOptions};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::SystemTime;
use typetag;

/// structure containing all resources that commands may need to access
pub struct CommandContext {
    pub loghandler: LogHandler,
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
    pub program: String,
    pub args: Vec<String>,
}

// implement the ability to create a CmdOnce from clap's ArgMatches
// consider moving this to a utility file or in sibyl.rs
impl From<&ArgMatches<'_>> for CmdOnce {
    fn from(matches: &ArgMatches) -> Self {
        let cmdline = matches.values_of("cmd").unwrap().collect::<Vec<_>>();

        let (program, args) = cmdline.split_at(1);

        CmdOnce {
            program: program[0].to_string(),
            args: args.to_vec().iter().map(|s| s.to_string()).collect(),
        }
    }
}

// implement LogName for Once since it requires the ability to create logfiles
impl LogName for CmdOnce {
    fn log_name(&self) -> PathBuf {
        let mut v: Vec<&str> = vec![&self.program];
        let mut args: Vec<&str> = self.args.iter().map(|s| s.as_str()).collect();
        v.append(&mut args);

        let mut path = PathBuf::new();
        path.push(v.join("_"));

        path
    }
}

#[typetag::serde]
impl Action for CmdOnce {
    fn execute(&self, _req: &Request, ctx: &mut CommandContext) -> Result<Response> {
        let output_file = ctx.loghandler.create_log(self)?.open()?;

        Command::new(&self.program)
            .args(&self.args)
            .stdout(Stdio::from(output_file))
            .stderr(Stdio::null())
            .spawn()
            .context("failed to spawn process")?;

        Ok(Response {
            msg: format!("successfully executed process: {}", &self.program),
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
