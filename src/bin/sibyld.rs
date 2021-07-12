extern crate anyhow;
extern crate dirs;
#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use sibyl::commands::CommandContext;
use sibyl::logging::LogHandler;
use sibyl::processing::ProcessHandler;
use sibyl::{Client, Request, Response};
use std::net::TcpListener;

fn main() -> Result<()> {
    // use environment variable SIBYL_LOG for loglevel settings
    env_logger::Builder::from_env("SIBYL_LOG").init();

    let listener =
        TcpListener::bind("127.0.0.1:52352").context("failed to create TCP listener")?;
    info!("bound to port 52352!");

    // get (or create, if it does not exist) the log directory
    let mut path = dirs::data_local_dir().unwrap();
    path.push("sibyllogs");

    let mut ctx = CommandContext {
        loghandler: LogHandler::new(&path),
        prochandler: ProcessHandler::new(),
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
                    }
                    Err(e) => {
                        error!("failed to receive request: {}", e);
                        continue;
                    }
                };

                let res = process_command(&req, &mut ctx);

                match client.send_response(&res) {
                    Ok(_) => {
                        debug!("sent response: {:?}", res);
                    }
                    Err(e) => {
                        error!("failed to send response: {}", e);
                        continue;
                    }
                }
            }
            Err(e) => warn!("connection failed: {}", e),
        }
    }

    Ok(())
}

fn process_command(req: &Request, ctx: &mut CommandContext) -> Response {
    match req.command.execute(req, ctx) {
        Ok(r) => r,
        Err(e) => {
            error!("failed to execute a command!");
            Response {
                msg: format!("an error occurred: {}", e),
            }
        }
    }
}
