extern crate anyhow;
extern crate dirs;
#[macro_use]
extern crate log;
extern crate lockfile;

use std::io::Write;
use std::net::TcpListener;
use anyhow::Result;
use sibyl::{Request, processing::process_command, logging::LogHandler};


fn main() -> Result<()> {
    // use environment variable SIBYL_LOG for loglevel settings
    env_logger::Builder::from_env("SIBYL_LOG").init();

    let listener = TcpListener::bind("127.0.0.1:52452")?;
    info!("bound to port 52452");
    
    // get (or create, if it does not exist) the log directory
    let mut path = dirs::data_local_dir().unwrap();
    path.push("sibyllogs");

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