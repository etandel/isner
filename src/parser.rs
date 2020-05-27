use std::error::Error;
use std::io::{BufRead, Lines};

use http::{Method, Request, Uri};
use http::request::Builder;

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
        .uri(path)
    )

}

pub fn parse_request(reader: &mut dyn BufRead) -> Result<Request<String>, Box<dyn Error>> {
    let mut lines: Lines<&mut dyn BufRead> = reader.lines();
    let builder = parse_request_line(&mut lines)?;
    let req = builder.body("".to_owned())?;

    Ok(req)
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use http::method::Method;
    use http::request::Request;

    use super::*;

    #[test]
    fn parse_request_with_no_headers_no_body()  -> Result<(), Box<dyn Error>>{
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

}
