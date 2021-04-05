use std::io::{BufRead, Write};
use std::sync::Arc;

use anyhow::Error;
use fehler::throws;
use http::status::StatusCode;

use super::parser::{parse_request, Request};

pub type Response = http::response::Response<Vec<u8>>;

pub trait Handler: Sync + Send  {
    fn handle_request(&self, request: &Request) -> Response;
}

fn get_response(reader: &mut dyn BufRead, handler: Arc<impl Handler>) -> Response {
    let res = parse_request(reader);
    match res {
        Ok(req) => handler.handle_request(&req),
        Err(_) => http::Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(vec![])
            .unwrap(),
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
    let headers = resp.headers();
    for (header_name, value) in headers {
        if !value.is_empty() {
            writer.write_all(header_name.as_ref())?;
            writer.write_all(b": ")?;
            writer.write_all(value.as_bytes())?;
            writer.write_all(b"\r\n")?;
        }
    }
    write!(writer, "Connection: close \r\n")?;
    write!(writer, "\r\n")?;

    writer.write_all(resp.body())?;
}

#[throws]
pub fn handle(
    reader: &mut dyn BufRead,
    writer: &mut dyn Write,
    handler: Arc<impl Handler>,
) -> Response {
    let response = get_response(reader, handler);
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

    struct OkEmptyHandler {}
    impl Handler for OkEmptyHandler {
        fn handle_request(&self, _request: &Request) -> Response {
            http::Response::builder()
                .status(StatusCode::OK)
                .header("Content-length", "0")
                .body(vec![])
                .unwrap()
        }
    }

    fn mock_reader(raw_request: &str) -> BufReader<&[u8]> {
        BufReader::new(raw_request.as_bytes())
    }

    #[test]
    fn get_response_ok() {
        let h = Arc::new(OkEmptyHandler {});
        let got = get_response(&mut mock_reader("GET / HTTP/1.0\r\n\r\n"), h);
        assert_eq!(got.status(), StatusCode::OK);
    }

    #[test]
    fn get_response_with_parse_error() {
        let h = Arc::new(OkEmptyHandler {});
        let got = get_response(&mut mock_reader(""), h);
        assert_eq!(got.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    #[throws]
    fn write_response_ok() {
        let resp = http::Response::builder()
            .status(StatusCode::OK)
            .body(vec![])?;
        let mut writer = Vec::new();
        write_response(&resp, &mut writer)?;
        let got = String::from_utf8(writer)?;
        assert_eq!(
            &got,
            "HTTP/1.1 200 OK\r\nConnection: close \r\n\r\n"
        );
    }

    #[test]
    #[throws]
    fn write_response_err() {
        let resp = http::Response::builder()
            .status(StatusCode::OK)
            .body(vec![])?;
        let mut writer = ErrWriter();
        let got = write_response(&resp, &mut writer);
        assert!(got.is_err(), "Error expected");
    }

    #[test]
    #[throws]
    fn handle_ok() {
        let h = Arc::new(OkEmptyHandler {});

        let raw_request = "GET / HTTP/1.0\r\n\r\n";

        let mut writer = Vec::new();
        let resp = handle(&mut mock_reader(&raw_request), &mut writer, h)?;
        let got = String::from_utf8(writer)?;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            &got,
            "HTTP/1.1 200 OK\r\ncontent-length: 0\r\nConnection: close \r\n\r\n"
        );
    }

    #[test]
    fn handle_write_err() {
        let h = Arc::new(OkEmptyHandler{});

        let raw_request = "GET / HTTP/1.0\r\n\r\n";

        let mut writer = ErrWriter();
        let got = handle(&mut mock_reader(&raw_request), &mut writer, h);
        assert!(got.is_err(), "Error expected");
    }
}
