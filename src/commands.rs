use std::path::PathBuf;
use clap::ArgMatches;
use serde::{Serialize, Deserialize};
use crate::logging::LogName;


// trait that describes any type of 
// command executable by the server
// the new function builds an action for sending 
// over the connection
pub trait Action {
    fn new(args: &ArgMatches) -> Self;
}

// enumeration over all possible commands
// that can be executed by the server
//
// this is built by a client and sent to the server
// in a Request to be executed
#[derive(Serialize, Deserialize, Debug)]
pub enum Cmd {
    Nop,
    Once(CmdOnce),
    Latest(CmdLatest),
}

// action that describes a program to be run once
// with output logged and stored in a temporary file
#[derive(Serialize, Deserialize, Debug)]
pub struct CmdOnce {
    pub program: String,
    pub args: Vec<String>,
}

impl Action for CmdOnce {
    fn new(matches: &ArgMatches) -> Self {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdLatest {}

impl Action for CmdLatest {
    fn new(_: &ArgMatches) -> Self {
        CmdLatest {}
    }
}