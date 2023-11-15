use super::{HttpRequest, HttpResponse};

///
/// Error type returned by failed ureq HTTP requests.
///
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Non-ureq HTTP error.
    #[error("HTTP error")]
    Http(#[from] http::Error),
    /// IO error
    #[error("IO error")]
    IO(#[from] std::io::Error),
    /// Other error.
    #[error("Other error: {}", _0)]
    Other(String),
    /// Error returned by ureq crate.
    // boxed due to https://github.com/algesten/ureq/issues/296
    #[error("ureq request failed")]
    Ureq(#[from] Box<ureq::Error>),
}

///
/// Synchronous HTTP client for ureq.
///
pub fn http_client(request: HttpRequest) -> Result<HttpResponse, Error> {
    let (parts, body) = request.into_parts();
    let request = ureq::Request::from(parts);
    let response = request.send_bytes(&body).unwrap();
    let response = HttpResponse::from(response);
    Ok(response)
}
