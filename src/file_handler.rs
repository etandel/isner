use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use log::debug;

use http::status::StatusCode;
use http::Method;

use super::http_handler::{Handler, Response};
use super::parser::Request;

pub struct FileHandler {
    root_dir: PathBuf,
}

impl FileHandler {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }
}

impl Handler for FileHandler {
    fn handle_request(&self, request: &Request) -> Response {
        if request.method() != Method::GET {
            return http::Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .header("Content-length", 0)
                .body(vec![])
                .unwrap();
        }
        let path = request.uri().path();
        let file_path = self.root_dir.join(path.trim_start_matches('/'));

        debug!("Got file_path: {:?}", file_path);
        match File::open(file_path) {
            Ok(mut file) => {
                let metadata = file.metadata().unwrap(); // TODO handle error
                let file_size = metadata.len();
                if !metadata.is_dir() {
                    let mut buff = Vec::with_capacity(file_size as usize);
                    let res = file.read_to_end(&mut buff);
                    debug!("{:?}", buff);
                    debug!("{:?}", res);
                    res.expect("Could not read file");
                    http::Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-length", file_size)
                        .header("Content-type", "text/plain") // TODO dehardcode
                        .body(buff)
                        .unwrap()
                } else {
                    http::Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(vec![])
                        .unwrap()
                }
            }
            Err(_) => http::Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(vec![])
                .unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Method;
    use std::path::Path;

    use anyhow::Error;
    use fehler::throws;

    struct TestFile {}
    impl TestFile {
        const EMPTY: &'static str = "empty_file.txt";
    }

    fn get_test_dir() -> PathBuf {
        Path::new("./test_data").to_owned()
    }

    fn get_uri_path(filename: &str) -> String {
        String::from("/") + filename
    }

    #[throws]
    fn build_request(method: Method, filename: &str) -> Request {
        Request::builder()
            .method(method)
            .uri(get_uri_path(filename))
            .body(())?
    }

    #[test]
    #[throws]
    fn handle_request_ok() {
        let request = build_request(Method::GET, TestFile::EMPTY)?;

        let h = FileHandler::new(get_test_dir());
        let got = h.handle_request(&request);
        assert_eq!(got.status(), StatusCode::OK);
    }

    #[test]
    #[throws]
    fn handle_request_method_not_allowed() {
        let h = FileHandler::new(get_test_dir());

        let request = build_request(Method::POST, TestFile::EMPTY)?;
        let got = h.handle_request(&request);
        assert_eq!(got.status(), StatusCode::METHOD_NOT_ALLOWED);

        let request = build_request(Method::PUT, TestFile::EMPTY)?;
        let got = h.handle_request(&request);
        assert_eq!(got.status(), StatusCode::METHOD_NOT_ALLOWED);

        let request = build_request(Method::PATCH, TestFile::EMPTY)?;
        let got = h.handle_request(&request);
        assert_eq!(got.status(), StatusCode::METHOD_NOT_ALLOWED);

        let request = build_request(Method::DELETE, TestFile::EMPTY)?;
        let got = h.handle_request(&request);
        assert_eq!(got.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
