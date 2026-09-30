//! A small HTTP/1.1 exchange with the backend proxy: one request per connection,
//! closed by the server when it is done.

use super::ServiceRequest;

/// The request bytes for `request`, to the proxy on `127.0.0.1:port`.
pub fn request_bytes(port: u16, request: &ServiceRequest) -> Vec<u8> {
    let body = request.body.as_deref().unwrap_or("");
    let mut head = format!(
        "{} {} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n",
        request.method, request.path
    );
    for (name, value) in &request.headers {
        let lower = name.to_ascii_lowercase();
        let reserved = matches!(
            lower.as_str(),
            "host" | "connection" | "content-length" | "transfer-encoding"
        );
        let clean = |text: &str| !text.contains(['\r', '\n']);
        if !reserved && clean(name) && clean(value) {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
    }
    if request.body.is_some() || !matches!(request.method.as_str(), "GET" | "HEAD") {
        head.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    head.push_str("\r\n");
    let mut bytes = head.into_bytes();
    bytes.extend_from_slice(body.as_bytes());
    bytes
}

/// The status and the body of a whole response. A chunked body is decoded.
pub fn parse_response(bytes: &[u8]) -> Result<(u16, String), String> {
    parse_response_parts(bytes).map(|response| (response.status, response.body))
}

/// A whole HTTP/1.1 response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    /// Header names and values in the order sent.
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl HttpResponse {
    /// The first header named `name`, ignoring case.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

/// The status, the headers, and the body of a whole response. A chunked body is
/// decoded.
pub fn parse_response_parts(bytes: &[u8]) -> Result<HttpResponse, String> {
    // Some small servers end lines with a bare LF.
    let (split, gap) = find(bytes, b"\r\n\r\n")
        .map(|at| (at, 4))
        .or_else(|| find(bytes, b"\n\n").map(|at| (at, 2)))
        .ok_or("the server sent no complete answer")?;
    let head = String::from_utf8_lossy(&bytes[..split]);
    let body = &bytes[split + gap..];
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .ok_or("the server's answer has no status")?;
    let headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_string(), value.trim().to_string()))
        .collect();
    let chunked = headers.iter().any(|(name, value)| {
        name.eq_ignore_ascii_case("transfer-encoding")
            && value.to_ascii_lowercase().contains("chunked")
    });
    let body = if chunked {
        dechunk(body)?
    } else {
        body.to_vec()
    };
    Ok(HttpResponse {
        status,
        headers,
        body: String::from_utf8_lossy(&body).into_owned(),
    })
}

fn dechunk(mut body: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    loop {
        let end = find(body, b"\r\n").ok_or("a chunk has no size")?;
        let size = String::from_utf8_lossy(&body[..end]);
        let size = size.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size, 16).map_err(|_| "a chunk size is not hex")?;
        body = &body[end + 2..];
        if size == 0 {
            return Ok(out);
        }
        let chunk = body.get(..size).ok_or("a chunk is cut short")?;
        out.extend_from_slice(chunk);
        body = body.get(size + 2..).unwrap_or_default();
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests;
