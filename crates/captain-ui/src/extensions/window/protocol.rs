//! Serves the extension's UI files on `captain-ext://<id>/`, so the page needs no
//! TCP port.

use std::borrow::Cow;
use std::path::Path;

use captain_core::extension::{mime_type, ui_file};
use wry::http::{Request, Response, StatusCode, header};

/// The file for `request` from `ui_dir`, if the URL's host is the extension `id`.
pub fn serve(ui_dir: &Path, id: &str, request: &Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    let uri = request.uri();
    if !uri.host().is_some_and(|host| host.eq_ignore_ascii_case(id)) {
        return status(StatusCode::NOT_FOUND);
    }
    let Some(mut file) = ui_file(ui_dir, uri.path()) else {
        return status(StatusCode::FORBIDDEN);
    };
    if file.is_dir() {
        file.push("index.html");
    }
    match std::fs::read(&file) {
        Ok(bytes) => Response::builder()
            .header(header::CONTENT_TYPE, mime_type(&file))
            .body(Cow::Owned(bytes))
            .unwrap_or_else(|_| status(StatusCode::INTERNAL_SERVER_ERROR)),
        Err(_) => status(StatusCode::NOT_FOUND),
    }
}

fn status(code: StatusCode) -> Response<Cow<'static, [u8]>> {
    let mut response = Response::new(Cow::Borrowed(&[][..]));
    *response.status_mut() = code;
    response
}
