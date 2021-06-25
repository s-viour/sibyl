extern crate anyhow;
#[macro_use]
extern crate clap;

use std::convert::From;
use anyhow::{Context, Result};
use clap::{App, ArgMatches};
use sibyl::{Request, Client};
use sibyl::commands::*;


fn main() -> Result<()> {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let req = match build_request(&matches) {
        Some(req) => req,
        None => {
            println!("no command specified");
            return Ok(())
        }
    };

    // this is a special case
    // we shouldn't even consider this a failure
    let mut client = match Client::connect() {
        Ok(client) => client,
        Err(_) => {
            // fail gracefully if the daemon isn't running
            println!("failed to establish link to sibyld");
            return Ok(());
        }
    };

    client.send_request(&req)
        .context("failed to send request to server")?;
    let res = client.receive_response()
        .context("failed to read response from daemon")?;
    println!("{}", res.msg);

    Ok(())
}

fn build_request(matches: &ArgMatches) -> Option<Request> {
    let command: Box<dyn Action>;
    
    if let Some(matches) = matches.subcommand_matches("once") {
        command = Box::new(CmdOnce::from(matches));
    } else if let Some(_) = matches.subcommand_matches("latest") {
        command = Box::new(CmdLatest)
    } else if let Some(_) = matches.subcommand_matches("ping") {
        command = Box::new(CmdPing)
    } else {
        return None;
    }

    Some(Request {
        command,
    })
}