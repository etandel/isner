use std::error::Error;
use std::io::{BufRead, Write};

use http::method::Method;
use http::request::Request;
use http::response::Response;

use super::parser::parse_request;

fn handle_request(req: &Request<()>) -> Response<()> {
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

fn get_response(reader: &mut dyn BufRead) -> Response<()> {
    let res = parse_request(reader);
    match res {
        Ok(req) => handle_request(&req),
        Err(_) => Response::builder().status(400).body(()).unwrap(),
    }
}

fn write_response(resp: &Response<()>, writer: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    write!(
        writer,
        "{} {}\r\n",
        resp.status().as_str(),
        resp.status().canonical_reason().unwrap_or("UNKNOWN")
    )?;
    write!(writer, "{}\r\n", "Connection: close")?;
    write!(writer, "{}\r\n", "Content-type: text/plain")?;
    write!(writer, "{}\r\n", "Content-length: 0")?;
    write!(writer, "\r\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn handle_request_ok() -> Result<(), Box<dyn Error>> {
        let request = Request::builder().method(Method::GET).uri("/").body(())?;
        let got = handle_request(&request);
        assert_eq!(got.status(), 200);
        Ok(())
    }

    #[test]
    fn handle_request_method_not_allowed() -> Result<(), Box<dyn Error>> {
        let request = Request::builder().method(Method::POST).uri("/").body(())?;
        let got = handle_request(&request);
        assert_eq!(got.status(), 405);
        Ok(())
    }

    #[test]
    fn get_response_with_parse_error() {
        let raw_request = "".as_bytes();
        let got = get_response(&mut BufReader::new(raw_request));
        assert_eq!(got.status(), 400);
    }

    #[test]
    fn write_response_ok() -> Result<(), Box<dyn Error>> {
        let resp = Response::builder().status(200).body(())?;
        let mut writer = Vec::new();
        write_response(&resp, &mut writer)?;
        let got = String::from_utf8(writer)?;
        assert_eq!(
            &got,
            "200 OK\r\nConnection: close\r\nContent-type: text/plain\r\nContent-length: 0\r\n\r\n"
        );
        Ok(())
    }

    use std::io;
    struct ErrWriter();
    impl Write for ErrWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::Other, "always fails"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Other, "always fails"))
        }
    }

    #[test]
    fn write_response_err() -> Result<(), Box<dyn Error>> {
        let resp = Response::builder().status(200).body(())?;
        let mut writer = ErrWriter();
        let got = write_response(&resp, &mut writer);
        assert!(got.is_err(), "Error expected");
        Ok(())
    }
}
