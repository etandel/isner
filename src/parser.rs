use std::error::Error;
use std::io::{BufRead, Lines};

use http::request::Builder;
use http::{Method, Request, Uri};

fn parse_request_line(lines: &mut Lines<&mut dyn BufRead>) -> Result<Builder, Box<dyn Error>> {
    let request_line = lines.next().ok_or("Empty request")??;
    let mut request_line_tokens = request_line.split(' ');
    let method = request_line_tokens.next().ok_or("Missing method")?;
    let path = request_line_tokens
        .next()
        .ok_or("Missing request path")?
        .parse::<Uri>()?;

    Ok(Request::builder()
        .method(method.parse::<Method>()?)
        .uri(path))
}

fn parse_headers(
    lines: &mut Lines<&mut dyn BufRead>,
    builder: Builder,
) -> Result<Builder, Box<dyn Error>> {
    let mut builder = builder;
    while let Some(header_line_result) = lines.next() {
        let header_line = header_line_result?;
        if header_line == "" {
            break;
        }

        let mut header_tokens = header_line.splitn(2, ": ");
        let key = header_tokens.next().ok_or("Missing header key")?;
        let value = header_tokens.next().ok_or("Missing header value")?;
        builder = builder.header(key, value);
    }

    Ok(builder)
}

pub fn parse_request(reader: &mut dyn BufRead) -> Result<Request<String>, Box<dyn Error>> {
    let mut lines: Lines<&mut dyn BufRead> = reader.lines();
    let builder = parse_request_line(&mut lines)?;
    let builder = parse_headers(&mut lines, builder)?;

    let req = builder.body("".to_owned())?;
    Ok(req)
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use core::convert::TryInto;
    use http::method::Method;
    use http::request::Request;

    use super::*;

    #[test]
    fn parse_request_with_no_headers_no_body() -> Result<(), Box<dyn Error>> {
        let raw_request = "GET /foo/bar HTTP/3.0".as_bytes();
        let got = parse_request(&mut BufReader::new(raw_request))?;
        let expected: Request<String> = Request::builder()
            .method(Method::GET)
            .uri("/foo/bar".as_bytes())
            .body("".to_owned())?;
        assert_eq!(got.method(), expected.method());
        assert_eq!(got.uri(), expected.uri());
        Ok(())
    }

    #[test]
    fn parse_request_with_headers_no_body() -> Result<(), Box<dyn Error>> {
        let raw_request = "GET /foo/bar HTTP/3.0\r\nfoo: bar\r\nfizz: buzz\r\n".as_bytes();
        let got = parse_request(&mut BufReader::new(raw_request))?;
        let expected: Request<String> = Request::builder()
            .method(Method::GET)
            .uri("/foo/bar".as_bytes())
            .header("foo", "bar")
            .header("fizz", "buzz")
            .body("".to_owned())?;
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
