use thiserror::Error;

///
/// Error type returned by failed reqwest HTTP requests.
///
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Error {
    /// Error returned by reqwest crate.
    #[error("request failed")]
    Reqwest(#[from] reqwest::Error),
    /// Non-reqwest HTTP error.
    #[error("HTTP error")]
    Http(#[from] http::Error),
}

#[cfg(not(target_arch = "wasm32"))]
pub use blocking::http_client;

pub use async_client::async_http_client;

#[cfg(not(target_arch = "wasm32"))]
mod blocking {

    use std::convert::TryFrom;

    use super::super::{HttpRequest, HttpResponse};
    use super::Error;

    pub use reqwest;
    use reqwest::blocking;
    use reqwest::redirect::Policy as RedirectPolicy;

    fn reqwest_to_http_response(response: blocking::Response) -> Result<HttpResponse, Error> {
        // TODO: Conversion to `http` types should happen upstream in `reqwest`: https://github.com/seanmonstar/reqwest/pull/1954#discussion_r1394232154
        // Ok(response.into())
        let mut builder = http::Response::builder()
            .status(response.status())
            .version(response.version());
        // TODO: Do this the other way around
        *builder.headers_mut().unwrap() = response.headers().clone();
        // TODO: The returned value should be infallible?
        let body = response.bytes()?.to_vec();
        Ok(builder.body(body)?)
    }

    ///
    /// Synchronous HTTP client.
    ///
    pub fn http_client(request: HttpRequest) -> Result<HttpResponse, Error> {
        let client = blocking::Client::builder()
            // Following redirects opens the client up to SSRF vulnerabilities.
            .redirect(RedirectPolicy::none())
            .build()?;

        let request = blocking::Request::try_from(request)?;
        let response = client.execute(request)?;
        reqwest_to_http_response(response)
    }
}

mod async_client {

    use std::convert::TryFrom;

    use super::super::{HttpRequest, HttpResponse};
    use super::Error;

    pub use reqwest;

    async fn reqwest_to_http_response(response: reqwest::Response) -> Result<HttpResponse, Error> {
        // TODO: Conversion to `http` types should happen upstream in `reqwest`: https://github.com/seanmonstar/reqwest/pull/1954#discussion_r1394232154
        // Ok(response.into())
        let mut builder = http::Response::builder()
            .status(response.status())
            .version(response.version());
        // TODO: Do this the other way around
        *builder.headers_mut().unwrap() = response.headers().clone();
        // TODO: The returned value should be infallible?
        let body = response.bytes().await?.to_vec();
        Ok(builder.body(body)?)
    }

    ///
    /// Asynchronous HTTP client.
    ///
    pub async fn async_http_client(request: HttpRequest) -> Result<HttpResponse, Error> {
        let client = {
            let builder = reqwest::Client::builder();

            // Following redirects opens the client up to SSRF vulnerabilities.
            // but this is not possible to prevent on wasm targets
            #[cfg(not(target_arch = "wasm32"))]
            let builder = builder.redirect(reqwest::redirect::Policy::none());

            builder.build()?
        };

        let request = reqwest::Request::try_from(request)?;

        let response = client.execute(request).await?;
        reqwest_to_http_response(response).await
    }
}
