use std::path::PathBuf;
use clap::ArgMatches;
use serde::{Serialize, Deserialize};
use crate::logging::LogName;
use typetag;


#[typetag::serde(tag = "type")]
pub trait Command {
    fn execute(&self);
}

// enumeration over all possible commands
// that can be executed by the server
//
// this is built by a client and sent to the server
// in a Request to be executed
#[derive(Serialize, Deserialize, Debug)]
pub enum Cmd {
    Once(CmdOnce),
    Latest,
    Ping,
}

// action that describes a program to be run once
// with output logged and stored in a temporary file
#[derive(Serialize, Deserialize, Debug)]
pub struct CmdOnce {
    pub program: String,
    pub args: Vec<String>,
}

impl From<&ArgMatches<'_>> for CmdOnce {
    fn from(matches: &ArgMatches) -> Self {
        let cmdline = matches.values_of("cmd")
            .unwrap()
            .collect::<Vec<_>>();

        
        let (program, args) = cmdline.split_at(1);
        
        CmdOnce {
            program: program[0].to_string(),
            args: args.to_vec().iter().map(|s| s.to_string()).collect(),
        }
    }
}

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
impl Command for CmdOnce {
    fn execute(&self) {
        println!("once command executed: {}", &self.program);
    }
}

#[derive(Serialize, Deserialize)]
pub struct CmdLatest;

#[typetag::serde]
impl Command for CmdLatest {
    fn execute(&self) {
        println!("latest command executed");
    }
}

#[derive(Serialize, Deserialize)]
pub struct CmdPing;

#[typetag::serde]
impl Command for CmdPing {
    fn execute(&self) {
        println!("ping commnad executed");
    }
}