use std::io;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use anyhow::Error;
use fehler::throws;
use log::{debug, info};
use threadpool::ThreadPool;

use crate::http_handler::{handle, Handler, Response};

#[throws]
fn handle_stream(stream: TcpStream, handler: Arc<impl Handler>) -> Response {
    let mut out = stream.try_clone()?;
    let mut reader = io::BufReader::new(stream);
    handle(&mut reader, &mut out, handler)?
}

fn uber_handle_stream(stream: io::Result<TcpStream>, handler: Arc<impl Handler>) {
    match stream {
        Ok(stream) => {
            let addr = stream.peer_addr().unwrap();
            debug!("Accepted connection: {}", addr);

            let res = handle_stream(stream, handler);
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

pub fn run(listener: TcpListener, pool: ThreadPool, handler: impl Handler + 'static) {
    info!(
        "Running server on {} with {} threads",
        listener.local_addr().unwrap(),
        pool.max_count()
    );
    let arch = Arc::new(handler);
    for stream in listener.incoming() {
        let ahandler = Arc::clone(&arch);

        pool.execute(move || {
            uber_handle_stream(stream, ahandler);
        });
    }
}
