extern crate anyhow;
extern crate chrono;
extern crate log;
extern crate serde;
extern crate typetag;

pub mod commands;
pub mod logging;

use anyhow::Result;
use chrono::{DateTime, Utc};
use commands::*;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

/// structure containing all information that *might* be required by the server to fufill a command
///
/// currently only contains a boxed Action trait
/// this structure is serialized using typetag
#[derive(Serialize, Deserialize)]
pub struct Request {
    pub command: Box<dyn Action>,
    pub time: DateTime<Utc>,
}

/// structure containing any information the client might report to the user
///
/// currently only contains a response message
#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub msg: String,
}

/// helper structure that represents a connection over a UnixStream (windows IPC not supported yet)
///
/// has convenience methods for sending and receiving requests and responses
pub struct Client {
    connection: UnixStream,
}

impl Client {
    /// connect to a unix socket in the hard-coded position as of right now
    ///
    /// returns a Result<Client>
    pub fn connect() -> Result<Client> {
        let connection = UnixStream::connect("/tmp/sibyl.sock")?;

        Ok(Client { connection })
    }

    /// creates a client by taking ownership of an already-existing UnixStream struct
    pub fn from_stream(connection: UnixStream) -> Client {
        Client { connection }
    }

    /// serialize and send a Request structure over the connection
    pub fn send_request(&mut self, msg: &Request) -> Result<()> {
        let serialized: Vec<u8> = bincode::serialize(&msg)?;
        send_reqres(&mut self.connection, &serialized)
    }

    /// serialize and send a Response structure over the connection
    pub fn send_response(&mut self, msg: &Response) -> Result<()> {
        let serialized: Vec<u8> = bincode::serialize(&msg)?;
        send_reqres(&mut self.connection, &serialized)
    }

    /// block and wait for a Request structure, then deserialize and return it
    pub fn receive_request(&mut self) -> Result<Request> {
        let received = read_reqres(&mut self.connection)?;
        Ok(bincode::deserialize(&received)?)
    }

    ///block and wait for a Response structure, then deserialize and return it
    pub fn receive_response(&mut self) -> Result<Response> {
        let received = read_reqres(&mut self.connection)?;
        Ok(bincode::deserialize(&received)?)
    }
}

/// helper function in this module for sending a request/response
///
/// # Arguments
/// * `stream` - the UnixStream to send the bytes over
/// * `msg` - a Vec of bytes to send
fn send_reqres(stream: &mut UnixStream, msg: &[u8]) -> Result<()> {
    let size: Vec<u8> = bincode::serialize(&msg.len())?;

    stream.write_all(&size)?;
    stream.write_all(&msg)?;

    Ok(())
}

/// helper function in this module for blocking and receiving a request/response
///
/// # Arguments
/// * `stream` - the UnixStream to read over
fn read_reqres(stream: &mut UnixStream) -> Result<Vec<u8>> {
    let mut size_buffer: [u8; 8] = [0; 8];
    stream.read_exact(&mut size_buffer)?;
    let size: usize = bincode::deserialize(&size_buffer)?;

    let mut buffer: Vec<u8> = Vec::new();
    buffer.resize(size, 0);
    stream.read_exact(buffer.as_mut_slice())?;

    Ok(buffer)
}
