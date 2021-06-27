extern crate anyhow;
extern crate ctrlc;
extern crate dirs;
#[macro_use]
extern crate log;


use std::os::unix::net::UnixListener;
use anyhow::{Context, Result};
use sibyl::{Client, Response};
use sibyl::commands::{Action, CommandContext};
use sibyl::logging::LogHandler;


fn main() -> Result<()> {
    // use environment variable SIBYL_LOG for loglevel settings
    env_logger::Builder::from_env("SIBYL_LOG").init();

    let listener = UnixListener::bind("/tmp/sibyl.sock")
        .context("failed to create listener socket")?;
    info!("created listener socket at /tmp/sibyl.sock!");

    // handle ctrlc by removing the socket file and quitting
    ctrlc::set_handler(move || {
        warn!("exiting via ctrl+c is not recommended!");

        std::fs::remove_file("/tmp/sibyl.sock")
            .expect("failed to remove socket file! did you delete it while the process was running?");
        std::process::exit(0);
    }).context("failed to set ctrlc handler!")?;
    
    // get (or create, if it does not exist) the log directory
    let mut path = dirs::data_local_dir().unwrap();
    path.push("sibyllogs");

    let mut ctx = CommandContext {
        loghandler: LogHandler::new(&path),
    };

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                // create a client for each incoming stream
                let mut client = Client::from_stream(stream);
                
                // match statement is here so we can handle failure gracefully
                let req = match client.receive_request() {
                    Ok(req) => {
                        info!("got request");
                        req
                    },
                    Err(e) => {
                        error!("failed to receive request: {}", e);
                        continue;
                    }
                };

                let res = process_command(&req.command, &mut ctx);
                
                match client.send_response(&res) {
                    Ok(_) => {
                        debug!("sent response: {:?}", res);
                    },
                    Err(e) => {
                        error!("failed to send response: {}", e);
                        continue;
                    }
                }
                
            },
            Err(e) => warn!("connection failed: {}", e),
        }
    }

    Ok(())
}

fn process_command(cmd: &Box<dyn Action>, ctx: &mut CommandContext) -> Response {
    match cmd.execute(ctx) {
        Ok(r) => r,
        Err(e) => {
            error!("failed to execute a command!");
            Response {
                msg: format!("an error occurred: {}", e),
            }
        }
    }
}