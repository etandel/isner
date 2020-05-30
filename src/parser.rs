use std::io::{BufRead, Lines};

use anyhow::{Context, Result};
use http::request::Builder;
use thiserror::Error;

pub type Request = http::Request<()>;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Empty request")]
    EmptyRequest,
    #[error("Missing method")]
    MissingMethod,
    #[error("Missing path")]
    MissingRequestPath,
    #[error("Missing header key")]
    MissingHeaderKey,
    #[error("Missing header value")]
    MissingHeaderValue,
    #[error("Invalid Request")]
    InvalidRequest,
}

fn parse_request_line(lines: &mut Lines<&mut dyn BufRead>) -> Result<Builder> {
    let request_line = lines.next().context(ParseError::EmptyRequest)??;
    let mut request_line_tokens = request_line.split(' ');
    let method = request_line_tokens
        .next()
        .context(ParseError::MissingMethod)?;
    let path = request_line_tokens
        .next()
        .context(ParseError::MissingRequestPath)?;

    Ok(Request::builder().method(method).uri(path))
}

fn parse_headers(lines: &mut Lines<&mut dyn BufRead>, builder: Builder) -> Result<Builder> {
    let mut builder = builder;
    for header_line_result in lines {
        let header_line = header_line_result?;
        if header_line == "" {
            break;
        }

        let mut header_tokens = header_line.splitn(2, ": ");
        let key = header_tokens.next().context(ParseError::MissingHeaderKey)?;
        let value = header_tokens
            .next()
            .context(ParseError::MissingHeaderValue)?;
        builder = builder.header(key, value);
    }

    Ok(builder)
}

pub fn parse_request(reader: &mut dyn BufRead) -> Result<Request> {
    let mut lines: Lines<&mut dyn BufRead> = reader.lines();
    let builder = parse_request_line(&mut lines)?;
    let builder = parse_headers(&mut lines, builder)?;

    let req = builder.body(()).context(ParseError::InvalidRequest)?;
    Ok(req)
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use core::convert::TryInto;
    use http::method::Method;

    use super::*;

    #[test]
    fn parse_request_with_no_headers_no_body() -> Result<()> {
        let raw_request = "GET /foo/bar HTTP/3.0".as_bytes();
        let got = parse_request(&mut BufReader::new(raw_request))?;
        let expected: Request = Request::builder()
            .method(Method::GET)
            .uri("/foo/bar".as_bytes())
            .body(())?;
        assert_eq!(got.method(), expected.method());
        assert_eq!(got.uri(), expected.uri());
        Ok(())
    }

    #[test]
    fn parse_request_with_headers_no_body() -> Result<()> {
        let raw_request = "GET /foo/bar HTTP/3.0\r\nfoo: bar\r\nfizz: buzz\r\n".as_bytes();
        let got = parse_request(&mut BufReader::new(raw_request))?;
        let expected: Request = Request::builder()
            .method(Method::GET)
            .uri("/foo/bar".as_bytes())
            .header("foo", "bar")
            .header("fizz", "buzz")
            .body(())?;
        assert_eq!(got.method(), expected.method());
        assert_eq!(got.uri(), expected.uri());

        let headers = got.headers();
        assert_eq!(headers.get("foo"), Some(&("bar".try_into()?)));
        assert_eq!(headers.get("fizz"), Some(&("buzz".try_into()?)));
        Ok(())
    }

    #[test]
    #[should_panic]
    fn parse_empty_request() {
        let raw_request = "".as_bytes();
        parse_request(&mut BufReader::new(raw_request)).unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_invalid_method() {
        let raw_request = "&&& /foo/bar".as_bytes();
        parse_request(&mut BufReader::new(raw_request)).unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_missing_path() {
        let raw_request = "GET ".as_bytes();
        parse_request(&mut BufReader::new(raw_request)).unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_invalid_path() {
        let raw_request = "GET \\".as_bytes();
        parse_request(&mut BufReader::new(raw_request)).unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_invalid_header_line() {
        let raw_request = "GET / HTTP/1.1\r\nfoo bar".as_bytes();
        parse_request(&mut BufReader::new(raw_request)).unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_invalid_header_name() {
        let raw_request = "GET / HTTP/1.1\r\n\\: bar".as_bytes();
        parse_request(&mut BufReader::new(raw_request)).unwrap();
    }
}
