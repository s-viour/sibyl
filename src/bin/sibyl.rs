extern crate anyhow;
#[macro_use]
extern crate clap;

use anyhow::{Context, Result};
use clap::{App, ArgMatches};
use sibyl::{Request, Client};
use sibyl::commands::*;


fn main() -> Result<()> {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

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
    
    match build_request(&matches) {
        Some(req) => client.send(&req)?,
        None =>  {
            println!("invalid command\n{}", matches.usage());
            return Ok(());
        }
    }


    let res = client.receive()
        .context("failed to read response from daemon")?;
    println!("{}", res.msg);

    Ok(())
}

fn build_request(matches: &ArgMatches) -> Option<Request> {
    let command: Cmd;
    
    if let Some(matches) = matches.subcommand_matches("once") {
        command = Cmd::Once(CmdOnce::new(matches));
    } else {
        return None;
    }

    Some(Request {
        command,
    })
}