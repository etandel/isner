use std::net::TcpListener;

use env_logger;

use rust_http_server::connection_handler::run;

fn main() {
    env_logger::init();
    let listener = TcpListener::bind("0.0.0.0:8000").expect("Could not bind to 0.0.0.0:8000");
    run(listener);
}
