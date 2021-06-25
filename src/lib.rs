extern crate anyhow;
extern crate log;
extern crate serde;
extern crate typetag;

pub mod commands;
pub mod logging;

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use anyhow::Result;
use bincode;
use serde::{Serialize, Deserialize};
use commands::*;


#[derive(Serialize, Deserialize)]
pub struct Request {
    pub command: Box<dyn Action>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub msg: String,
}


pub struct Client {
    connection: UnixStream,
}

impl Client {
    pub fn connect() -> Result<Client> {
        let connection = UnixStream::connect("/tmp/sibyl.sock")?;

        Ok(Client {
            connection,
        })
    }

    pub fn from_stream(connection: UnixStream) -> Client {
        Client {
            connection,
        }
    }

    pub fn send_request(&mut self, msg: &Request) -> Result<()> {
        let serialized: Vec<u8> = bincode::serialize(&msg)?;
        send_reqres(&mut self.connection, &serialized)
    }

    pub fn send_response(&mut self, msg: &Response) -> Result<()> {
        let serialized: Vec<u8> = bincode::serialize(&msg)?;
        send_reqres(&mut self.connection, &serialized)
    }

    pub fn receive_request(&mut self) -> Result<Request> {
        let received = read_reqres(&mut self.connection)?;
        Ok(bincode::deserialize(&received)?)
    }

    pub fn receive_response(&mut self) -> Result<Response> {
        let received = read_reqres(&mut self.connection)?;
        Ok(bincode::deserialize(&received)?)
    }
}

fn send_reqres(stream: &mut UnixStream, msg: &Vec<u8>) -> Result<()> {
    let size: Vec<u8> = bincode::serialize(&msg.len())?;

    stream.write(&size)?;
    stream.write(&msg)?;

    Ok(())
}

fn read_reqres(stream: &mut UnixStream) -> Result<Vec<u8>> {
    let mut size_buffer: [u8; 8] = [0; 8];
    stream.read(&mut size_buffer)?;
    let size: usize = bincode::deserialize(&size_buffer)?;

    let mut buffer: Vec<u8> = Vec::new();
    buffer.resize(size, 0);
    stream.read(buffer.as_mut_slice())?;

    Ok(buffer)
}