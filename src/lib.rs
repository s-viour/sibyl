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

//const SERDE_SIZE: usize = 4096;

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub command: Cmd,
}

impl Request {
    pub fn read(stream: &mut TcpStream) -> Result<Request> {
        let data = read_reqres(stream)?;
        Ok(bincode::deserialize(&data)?)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub msg: String,
}

impl Response {
    pub fn read(stream: &mut TcpStream) -> Result<Response> {
        let data = read_reqres(stream)?;
        Ok(bincode::deserialize(&data)?)
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
        let size: Vec<u8> = bincode::serialize(&serialized.len())?;
        
        &self.connection.write(&size)?;
        &self.connection.write(&serialized)?;

        Ok(())
    }

    pub fn receive(&mut self) -> Result<Response> {
        Response::read(&mut self.connection)
    }
}

pub fn send_response(conn: &mut TcpStream, res: &Response) -> Result<()> {
    let serialized: Vec<u8> = bincode::serialize(&res)?;
    let size: Vec<u8> = bincode::serialize(&serialized.len())?;
    
    conn.write(&size)?;
    conn.write(&serialized)?;

    Ok(())
}

fn read_reqres(stream: &mut TcpStream) -> Result<Vec<u8>> {
    let mut size_buffer: [u8; 8] = [0; 8];
    stream.read(&mut size_buffer)?;
    let size: usize = bincode::deserialize(&size_buffer)?;

    let mut buffer: Vec<u8> = Vec::new();
    buffer.resize(size, 0);
    stream.read(buffer.as_mut_slice())?;

    Ok(buffer)
}