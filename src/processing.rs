use std::convert::From;
use std::fs::{OpenOptions, metadata, read_dir};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::SystemTime;
use anyhow::{Context, Result};
use log::{info};
use crate::Response;
use crate::commands::*;
use crate::logging::LogHandler;

pub fn process_command(log_handler: &mut LogHandler, cmd: &Cmd) -> Response {
    match cmd {
        Cmd::Once(info) => match once(log_handler, &info) {
            Ok(r) => r,
            Err(e) => {
                info!("failed to execute a Once command");
                Response {
                    msg: format!("error: {}", e),
                }
            }
        },
        Cmd::Latest => match latest(log_handler) {
            Ok(r) => r,
            Err(e) => {
                info!("failed to execute a Latest command");
                Response {
                    msg: format!("error: {}", e),
                }
            }
        }
        Cmd::Ping => Response { msg: "pong!".to_string() },
    }
}

fn once(log_handler: &mut LogHandler, info: &CmdOnce) -> Result<Response> {
    let output_file = log_handler.create_log(info)?.open()?;

    Command::new(&info.program)
        .args(&info.args)
        .stdout(Stdio::from(output_file))
        .stderr(Stdio::null())
        .spawn()
        .context("failed to spawn process")?;

    Ok(Response {
        msg: format!("successfully executed process: {}", &info.program),
    })
}

fn latest(log_handler: &mut LogHandler) -> Result<Response> {
    let path = log_handler.log_directory();
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

    Ok(Response {
        msg: s,
    })
}