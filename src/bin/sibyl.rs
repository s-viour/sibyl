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
    
    let req = build_request(&matches);
    client.send(&req)
        .context("failed to make request to daemon")?;


    let res = client.receive()
        .context("failed to read response from daemon")?;
    println!("{}", res.msg);

    Ok(())
}

fn build_request(matches: &ArgMatches) -> Request {
    let command: Cmd;
    
    if let Some(matches) = matches.subcommand_matches("once") {
        command = Cmd::Once(CmdOnce::new(matches));
    } else {
        command = Cmd::Nop;
    }

    Request {
        command,
    }
}