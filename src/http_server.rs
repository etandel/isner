use std::io::BufRead;

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

#[cfg(test)]
mod tests {
    use std::error::Error;
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
    #[should_panic]
    fn get_response_with_parse_error() {
        let raw_request = "".as_bytes();
        let got = get_response(&mut BufReader::new(raw_request));
        assert_eq!(got.status(), 404);
    }
}
