//! HTTP module for runestick based on reqwest.
//!
//! ## Usage
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! runestick = "0.2"
//! runestick-http = "0.2"
//! # not necessary, but useful
//! runestick-json = "0.2"
//! ```
//!
//! Install it into your context:
//!
//! ```rust
//! # fn main() -> runestick::Result<()> {
//! let mut context = runestick::Context::with_default_packages()?;
//! context.install(runestick_http::module()?)?;
//! context.install(runestick_json::module()?)?;
//! # Ok(())
//! # }
//! ```
//!
//! Use it in Rune:
//!
//! ```rust,ignore
//! use http;
//! use json;
//!
//! fn main() {
//!     let client = http::Client::new();
//!     let response = client.get("http://worldtimeapi.org/api/ip");
//!     let text = response.text();
//!     let json = json::from_string(text);
//!
//!     let timezone = json["timezone"];
//!
//!     if timezone is String {
//!         dbg(timezone);
//!     }
//!
//!     let body = json::to_bytes(#{"hello": "world"});
//!
//!     let response = client.post("https://postman-echo.com/post")
//!         .body_bytes(body)
//!         .send();
//!
//!     let response = json::from_string(response.text());
//!     dbg(response);
//! }
//! ```

use runestick::Bytes;
use std::fmt;
use std::fmt::Write as _;

#[derive(Debug)]
pub struct Error {
    inner: reqwest::Error,
}

impl From<reqwest::Error> for Error {
    fn from(inner: reqwest::Error) -> Self {
        Self { inner }
    }
}

#[derive(Debug)]
struct Client {
    client: reqwest::Client,
}

#[derive(Debug)]
pub struct Response {
    response: reqwest::Response,
}

#[derive(Debug)]
pub struct StatusCode {
    inner: reqwest::StatusCode,
}

impl StatusCode {
    fn display(&self, buf: &mut String) -> fmt::Result {
        write!(buf, "{}", self.inner)
    }
}

impl Response {
    async fn text(self) -> Result<String, Error> {
        let text = self.response.text().await?;
        Ok(text)
    }

    /// Get the status code of the response.
    fn status(&self) -> StatusCode {
        let inner = self.response.status();

        StatusCode { inner }
    }
}

#[derive(Debug)]
pub struct RequestBuilder {
    request: reqwest::RequestBuilder,
}

impl RequestBuilder {
    /// Send the request being built.
    async fn send(self) -> Result<Response, Error> {
        let response = self.request.send().await?;
        Ok(Response { response })
    }

    async fn body_bytes(self, bytes: Bytes) -> Result<Self, Error> {
        let bytes = bytes.into_vec();

        Ok(Self {
            request: self.request.body(bytes),
        })
    }
}

impl Client {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Construct a builder to GET the given URL.
    async fn get(&self, url: &str) -> Result<RequestBuilder, Error> {
        let request = self.client.get(url);
        Ok(RequestBuilder { request })
    }

    /// Construct a builder to POST to the given URL.
    async fn post(&self, url: &str) -> Result<RequestBuilder, Error> {
        let request = self.client.post(url);
        Ok(RequestBuilder { request })
    }
}

/// Shorthand for generating a get request.
async fn get(url: &str) -> Result<Response, Error> {
    Ok(Response {
        response: reqwest::get(url).await?,
    })
}

runestick::decl_external!(Error);
runestick::decl_external!(Client);
runestick::decl_external!(Response);
runestick::decl_external!(RequestBuilder);
runestick::decl_external!(StatusCode);

/// Construct the http library.
pub fn module() -> Result<runestick::Module, runestick::ContextError> {
    let mut module = runestick::Module::new(&["http"]);

    module.ty(&["Client"]).build::<Client>()?;
    module.ty(&["Response"]).build::<Response>()?;
    module.ty(&["RequestBuilder"]).build::<RequestBuilder>()?;
    module.ty(&["StatusCode"]).build::<StatusCode>()?;
    module.ty(&["Error"]).build::<Error>()?;

    module.function(&["Client", "new"], Client::new)?;
    module.async_function(&["get"], get)?;

    module.async_inst_fn("get", Client::get)?;
    module.async_inst_fn("post", Client::post)?;

    module.async_inst_fn("text", Response::text)?;
    module.inst_fn("status", Response::status)?;

    module.async_inst_fn("send", RequestBuilder::send)?;
    module.async_inst_fn("body_bytes", RequestBuilder::body_bytes)?;

    module.inst_fn(runestick::FMT_DISPLAY, StatusCode::display)?;
    Ok(module)
}
