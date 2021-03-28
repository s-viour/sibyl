extern crate anyhow;
#[macro_use]
extern crate log;
extern crate lockfile;

use std::io::Write;
use std::net::TcpListener;
use std::path::Path;
use anyhow::Result;
use sibyl::{Request, processing::process_command, logging::LogHandler};


fn main() -> Result<()> {
    // TODO: set env logger to read from environment variable SIBYL_LOG
    env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:52452")?;
    info!("bound to port 52452");
    // TODO: change this to actually be a reasonable directory
    let path = Path::new("/tmp/sibyllog");
    let mut log_handler = LogHandler::new(&path);

    for connection in listener.incoming() {
        match connection {
            Ok(mut stream) => {
                let got = Request::read(&mut stream)?;
                info!("got request: {:?}", got);

                let res = process_command(&mut log_handler, &got.command);

                stream.write(&bincode::serialize(&res)?)?;
                debug!("sent response: {:?}", res);
            },
            Err(e) => warn!("connection failed: {}", e),
        }
    }

    Ok(())
}