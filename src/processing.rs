use std::convert::From;
use std::process::{Command, Stdio};
use anyhow::{Context, Result};
use log::{info};
use crate::Response;
use crate::commands::*;
use crate::logging::LogHandler;

pub fn process_command(log_handler: &mut LogHandler, cmd: &Cmd) -> Response {
    match cmd {
        Cmd::Nop => Response {
            msg: "invalid command!".to_string(),
        },
        Cmd::Once(info) => match once(log_handler, &info) {
            Ok(r) => r,
            Err(e) => {
                info!("failed to execute a Once command");
                Response {
                    msg: format!("error: {}", e),
                }
            }
        }
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