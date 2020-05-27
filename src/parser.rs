use std::io::BufRead;

use http::{Method, Request, Uri};

fn parse_request(reader: &mut dyn BufRead) -> Request<String> {
    let mut lines = reader.lines();
    let request_line = lines.next().unwrap().unwrap();
    let mut request_line_tokens = request_line.split(' ');
    let method = request_line_tokens.next().unwrap();
    let path = request_line_tokens.next().unwrap().parse::<Uri>().unwrap();
    Request::builder()
        .method(method.parse::<Method>().unwrap())
        .uri(path)
        .body("".to_owned())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read};

    use http::method::Method;
    use http::request::{Builder, Request};

    use super::*;

    #[test]
    fn read_request__no_headers__no_body() {
        let raw_request = "GET /foo/bar HTTP/3.0".as_bytes();
        let got = parse_request(&mut BufReader::new(raw_request));
        let expected: Request<String> = Request::builder()
            .method(Method::GET)
            .uri("/foo/bar".as_bytes())
            .body("".to_owned())
            .unwrap();
        assert_eq!(got.method(), expected.method());
        assert_eq!(got.uri(), expected.uri());
    }
}
