use std::io;
use std::net::{TcpListener, TcpStream};

use anyhow::Error;
use fehler::throws;
use log::{debug, info};
use threadpool::ThreadPool;

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

pub fn run(listener: TcpListener, pool: ThreadPool) {
    info!(
        "Running server on {} with {} threads",
        listener.local_addr().unwrap(),
        pool.max_count()
    );
    for stream in listener.incoming() {
        pool.execute(move || {
            uber_handle_stream(stream);
        });
    }
}
