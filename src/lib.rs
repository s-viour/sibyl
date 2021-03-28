extern crate anyhow;
extern crate log;
extern crate serde;

pub mod commands;
pub mod logging;
pub mod processing;

use std::io::{Read, Write};
use std::net::TcpStream;
use anyhow::Result;
use bincode;
use serde::{Serialize, Deserialize};
use commands::*;

const SERDE_SIZE: usize = 4096;


#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub command: Cmd,
}

impl Request {
    pub fn read(stream: &mut TcpStream) -> Result<Request> {
        let mut buffer: [u8; SERDE_SIZE] = [0; SERDE_SIZE];
        stream.read(&mut buffer)?;

        Ok(bincode::deserialize(&buffer)?)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub msg: String,
}

impl Response {
    pub fn read(stream: &mut TcpStream) -> Result<Response> {
        let mut buffer: [u8; SERDE_SIZE] = [0; SERDE_SIZE];
        stream.read(&mut buffer)?;

        Ok(bincode::deserialize(&buffer)?)
    }
}

// we could totally think about just moving all this code
// directly to the client program
// since i don't really suppose it's necessary to 
// abstract away like this
pub struct Client {
    connection: TcpStream,
}

impl Client {
    pub fn connect() -> Result<Client> {
        let connection = TcpStream::connect("127.0.0.1:52452")?;

        Ok(Client {
            connection,
        })
    }

    pub fn send(&mut self, msg: &Request) -> Result<()> {
        let serialized: Vec<u8> = bincode::serialize(&msg)?;
        
        &self.connection.write(&serialized)?;

        Ok(())
    }

    pub fn receive(&mut self) -> Result<Response> {
        let mut buffer: [u8; SERDE_SIZE] = [0; SERDE_SIZE];

        &self.connection.read(&mut buffer)?;
        
        Ok(bincode::deserialize(&buffer)?)
    }
}