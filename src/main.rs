use std::net::{TcpListener, ToSocketAddrs};
use std::path::Path;

use anyhow::{Context, Error};
use clap::{App, Arg};
use fehler::throws;
use threadpool::ThreadPool;

use isner::connection_handler::run;
use isner::file_handler::FileHandler;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "8000";
const DEFAULT_CONCURRENCY: &str = "10";

fn build_args<'a>() -> App<'a, 'a> {
    App::new("isner")
        .version("0.0.1-alpha.1")
        .author("Elias Tandel <elias.tandel@gmail.com>")
        .about("Simple file server in rust")
        .arg(
            Arg::with_name("host")
                .help("Address to which the server will be bound")
                .short("h")
                .long("host")
                .value_name("ADDRESS")
                .default_value(DEFAULT_HOST),
        )
        .arg(
            Arg::with_name("port")
                .help("Port to which the server will be bound")
                .short("p")
                .long("port")
                .value_name("PORT")
                .default_value(DEFAULT_PORT),
        )
        .arg(
            Arg::with_name("concurrency")
                .help("Maximum number of concurrent requests")
                .short("c")
                .long("concurrency")
                .default_value(DEFAULT_CONCURRENCY),
        )
}

#[throws]
fn main() {
    env_logger::init();

    let matches = build_args().get_matches();

    let listener = {
        let host = matches.value_of("host").unwrap_or(DEFAULT_HOST);
        let port = matches
            .value_of("port")
            .unwrap_or(DEFAULT_PORT)
            .parse::<u16>()?;
        let mut addrs_iter = (host, port).to_socket_addrs().context("Invalid address")?;
        let bind_to = addrs_iter.next().context("Invalid address")?;

        TcpListener::bind(bind_to).with_context(|| format!("Could not bind to {}", bind_to))?
    };

    let pool = {
        let pool_size = matches
            .value_of("concurrency")
            .unwrap_or(DEFAULT_CONCURRENCY)
            .parse::<usize>()
            .context("concurrency value must be integer")?;
        ThreadPool::new(pool_size)
    };

    let handler = FileHandler::new(Path::new("/tmp/www/").to_owned());

    run(listener, pool, handler);
}
