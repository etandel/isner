use std::io::{BufRead, Write};

use anyhow::Error;
use fehler::throws;
use http::method::Method;

use super::parser::{parse_request, Request};

pub type Response = http::response::Response<()>;

fn handle_request(req: &Request) -> Response {
    let builder = Response::builder();

    builder
        .status(if req.method() == Method::GET {
            200
        } else {
            405
        })
        .body(())
        .unwrap()
}

fn get_response(reader: &mut dyn BufRead) -> Response {
    let res = parse_request(reader);
    match res {
        Ok(req) => handle_request(&req),
        Err(_) => Response::builder().status(400).body(()).unwrap(),
    }
}

#[throws]
fn write_response(resp: &Response, writer: &mut dyn Write) {
    write!(
        writer,
        "HTTP/1.1 {} {}\r\n",
        resp.status().as_str(),
        resp.status().canonical_reason().unwrap_or("UNKNOWN")
    )?;
    write!(writer, "Connection: close\r\n")?;
    write!(writer, "Content-type: text/plain\r\n")?;
    write!(writer, "Content-length: 0\r\n")?;
    write!(writer, "\r\n")?;
}

#[throws]
pub fn handle(reader: &mut dyn BufRead, writer: &mut dyn Write) -> Response {
    let response = get_response(reader);
    write_response(&response, writer)?;
    response
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::BufReader;

    use super::*;

    struct ErrWriter();
    impl Write for ErrWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::Other, "always fails"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Other, "always fails"))
        }
    }

    fn mock_reader(raw_request: &str) -> BufReader<&[u8]> {
        BufReader::new(raw_request.as_bytes())
    }

    #[test]
    #[throws]
    fn handle_request_ok() {
        let request = Request::builder().method(Method::GET).uri("/").body(())?;
        let got = handle_request(&request);
        assert_eq!(got.status(), 200);
    }

    #[test]
    #[throws]
    fn handle_request_method_not_allowed() {
        let request = Request::builder().method(Method::POST).uri("/").body(())?;
        let got = handle_request(&request);
        assert_eq!(got.status(), 405);
    }

    #[test]
    fn get_response_ok() {
        let got = get_response(&mut mock_reader("GET / HTTP/1.0\r\n\r\n"));
        assert_eq!(got.status(), 200);
    }

    #[test]
    fn get_response_with_parse_error() {
        let got = get_response(&mut mock_reader(""));
        assert_eq!(got.status(), 400);
    }

    #[test]
    #[throws]
    fn write_response_ok() {
        let resp = Response::builder().status(200).body(())?;
        let mut writer = Vec::new();
        write_response(&resp, &mut writer)?;
        let got = String::from_utf8(writer)?;
        assert_eq!(
            &got,
            "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-type: text/plain\r\nContent-length: 0\r\n\r\n"
        );
    }

    #[test]
    #[throws]
    fn write_response_err() {
        let resp = Response::builder().status(200).body(())?;
        let mut writer = ErrWriter();
        let got = write_response(&resp, &mut writer);
        assert!(got.is_err(), "Error expected");
    }

    #[test]
    #[throws]
    fn handle_ok() {
        let raw_request = "GET / HTTP/1.0\r\n\r\n";

        let mut writer = Vec::new();
        let resp = handle(&mut mock_reader(&raw_request), &mut writer)?;
        let got = String::from_utf8(writer)?;

        assert_eq!(
            &got,
            "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-type: text/plain\r\nContent-length: 0\r\n\r\n"
        );
        assert_eq!(resp.status(), 200);
    }

    #[test]
    fn handle_write_err() {
        let raw_request = "GET / HTTP/1.0\r\n\r\n";

        let mut writer = ErrWriter();
        let got = handle(&mut mock_reader(&raw_request), &mut writer);
        assert!(got.is_err(), "Error expected");
    }
}
