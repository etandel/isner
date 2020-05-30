use std::io;
use std::net::{TcpListener, TcpStream};
use std::thread;

use anyhow::Error;
use fehler::throws;
use log::{debug, info};

use crate::http_handler::{handle, Response};

#[throws]
fn handle_stream(stream: TcpStream) -> Response {
    let mut out = stream.try_clone()?;
    let mut reader = io::BufReader::new(stream);
    handle(&mut reader, &mut out)?
}

fn uber_handle_stream(stream: io::Result<TcpStream>) {
    match stream {
        Ok(stream) => {
            let addr = stream.peer_addr().unwrap();
            debug!("Accepted connection: {}", addr);

            let res = handle_stream(stream);
            match res {
                Ok(response) => info!("{} {}", addr, response.status()),
                Err(e) => info!("An error happened while handling request {}", e),
            }
        }
        Err(e) => {
            info!("Connection failed: {}", e);
        }
    }
}

pub fn run(listener: TcpListener) {
    for stream in listener.incoming() {
        thread::spawn(move || {
            uber_handle_stream(stream);
        });
    }
}
