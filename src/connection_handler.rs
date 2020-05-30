use std::io;
use std::error::Error;
use std::net::{TcpListener, TcpStream};
use std::thread;

use http::Response;

use crate::http_handler::handle;


fn handle_stream(stream: TcpStream) -> Result<Response<()>, Box<dyn Error>> {
    let mut out = stream.try_clone()?;
    let mut reader = io::BufReader::new(stream);
    handle(&mut reader, &mut out)
}

fn uber_handle_stream(stream: io::Result<TcpStream>) {
    match stream {
        Ok(stream) => {
            let addr = stream.peer_addr().unwrap();
            eprintln!("Accepted connection: {}", addr);

            let res = handle_stream(stream);
            match res {
                Ok(response) => eprintln!("{} {}", addr, response.status()),
                Err(e) => eprintln!("An error happened while handling request {}", e),
            }
        }
        Err(e) => {
            eprintln!("Connection failed: {}", e);
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
