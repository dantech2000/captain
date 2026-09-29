use std::collections::BTreeMap;

use super::{parse_response, request_bytes};
use crate::extension::ServiceRequest;

#[test]
fn a_request_carries_its_body_and_drops_injected_headers() {
    let request = ServiceRequest {
        method: "POST".into(),
        path: "/items".into(),
        headers: BTreeMap::from([
            ("Content-Type".into(), "application/json".into()),
            ("X-Bad".into(), "a\r\nHost: evil".into()),
        ]),
        body: Some("{}".into()),
    };
    let text = String::from_utf8(request_bytes(4000, &request)).unwrap();
    assert!(text.starts_with("POST /items HTTP/1.1\r\nHost: 127.0.0.1:4000\r\n"));
    assert!(text.contains("Content-Type: application/json\r\n"));
    assert!(!text.contains("evil"));
    assert!(text.ends_with("Content-Length: 2\r\n\r\n{}"));
}

#[test]
fn a_response_body_may_be_chunked() {
    let plain = b"HTTP/1.1 404 Not Found\r\nContent-Length: 4\r\n\r\ngone";
    assert_eq!(parse_response(plain), Ok((404, "gone".into())));
    let chunked =
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n";
    assert_eq!(parse_response(chunked), Ok((200, "hello world".into())));
    assert_eq!(
        parse_response(b"HTTP/1.0 200 OK\n\nbare"),
        Ok((200, "bare".into()))
    );
    assert!(parse_response(b"HTTP/1.1 200").is_err());
}
