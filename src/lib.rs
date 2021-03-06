//! A simple HTTP request client library.
//!
//! #Example
//!
//! ```rust
//! # use std::io::Read;
//! let mut res = String::new();
//!
//! reru::post("https://httpbin.org/post")
//!     .expect("failed to parse URL")
//!     .param("show_env", "1")
//!     .body_json(&["蟹", "Ferris"])
//!     .expect("failed to serialize")
//!     .request()
//!     .expect("failed to send request")
//!     .read_to_string(&mut res)
//!     .expect("failed to read response");
//!
//! println!("{}", res);
//! ```

extern crate url;
extern crate hyper;

#[cfg(feature = "json")]
extern crate serde;
#[cfg(feature = "json")]
extern crate serde_json;

use url::Url;
use url::form_urlencoded::Serializer;
use hyper::header::{Headers, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::status::StatusCode;
use hyper::version::HttpVersion;
use hyper::client::{Client, IntoUrl};
use hyper::client::Response as HyperResponse;
use hyper::method::Method;
use hyper::error::Result as HyperResult;

#[cfg(feature = "json")]
use serde::ser::Serialize;
#[cfg(feature = "json")]
use serde::de::Deserialize;
#[cfg(feature = "json")]
use serde_json::error::Error as SerdeError;

/// A request.
#[derive(Clone, Debug)]
pub struct Request {
    pub method: Method,
    pub url: Url,
    pub headers: Headers,
    body: Body,
}

impl Request {
    /// Creates a new request.
    pub fn new<U: IntoUrl>(method: Method, url: U) -> Result<Request, url::ParseError> {
        Ok(Request {
            method: method,
            url: try!(url.into_url()),
            headers: Headers::new(),
            body: Body::None,
        })
    }

    /// Adds a name/value pair to URL's query string
    pub fn param(mut self, name: &str, value: &str) -> Self {
        self.url.query_pairs_mut().append_pair(name, value);
        self
    }

    /// Serializes a value as JSON, and sets it as HTTP POST data.
    /// By calling `body_json`, `Content-Type` of this request becomes
    /// `application/json`.
    #[cfg(feature = "json")]
    pub fn body_json<T: Serialize>(mut self, value: &T) -> Result<Self, serde_json::error::Error> {
        self.body = Body::Buffer(try!(serde_json::to_vec(value)));
        self.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
        Ok(self)
    }

    /// Adds a key/value pair for HTTP POST data.
    /// By calling `body_form`, `Content-Type` of this request becomes
    /// `application/x-www-form-urlencoded`.
    pub fn body_form(mut self, name: &str, value: &str) -> Self {
        self.body = Body::Forms(match self.body {
            Body::None | Body::Buffer(_) => {
                self.headers
                    .set(ContentType(Mime(TopLevel::Application,
                                          SubLevel::WwwFormUrlEncoded,
                                          vec![])));

                vec![(name.to_string(), value.to_string())]
            }
            Body::Forms(mut v) => {
                v.push((name.to_string(), value.to_string()));
                v
            }
        });

        self
    }

    /// Executes this request.
    pub fn request(self) -> HyperResult<Response> {
        self.request_with_client(Client::new())
    }

    /// Executes this request with a supplied `Client`.
    pub fn request_with_client(self, client: Client) -> HyperResult<Response> {
        // let c = client.request(..) <-- This outlives `encoded`

        Ok(Response::new(try!(match self.body {
            Body::Buffer(ref body) => {
                client.request(self.method, self.url)
                    .headers(self.headers)
                    .body(hyper::client::Body::BufBody(&body, body.len()))
                    .send()
            }

            Body::Forms(v) => {
                let mut ser = Serializer::new(String::new());

                for (n, v) in v {
                    ser.append_pair(&n, &v);
                }

                let encoded = ser.finish();
                client.request(self.method, self.url)
                    .headers(self.headers)
                    .body(hyper::client::Body::BufBody(encoded.as_bytes(), encoded.len()))
                    .send()
            }

            Body::None => {
                client.request(self.method, self.url)
                    .headers(self.headers)
                    .send()
            }
        })))
    }
}

#[derive(Clone, Debug)]
enum Body {
    None,
    Buffer(Vec<u8>),
    Forms(Vec<(String, String)>),
}

/// A response for a request. This is a wrapper around
/// hyper's [`Response`](../hyper/client/response/struct.Response.html) struct for JSON deserialization support.
#[derive(Debug)]
pub struct Response {
    hyper_response: HyperResponse,
}

impl Response {
    /// Wraps hyper's [`Response`](../hyper/client/response/struct.Response.html) struct.
    #[inline]
    pub fn new(hyper_response: HyperResponse) -> Response {
        Response { hyper_response: hyper_response }
    }

    #[inline]
    pub fn status(&self) -> &StatusCode {
        &self.hyper_response.status
    }

    #[inline]
    pub fn headers(&self) -> &Headers {
        &self.hyper_response.headers
    }

    #[inline]
    pub fn version(&self) -> &HttpVersion {
        &self.hyper_response.version
    }

    #[inline]
    pub fn url(&self) -> &Url {
        &self.hyper_response.url
    }

    /// Deserializes this response's body as a JSON.
    #[cfg(feature = "json")]
    pub fn parse_json<T: Deserialize>(self) -> Result<T, SerdeError> {
        Ok(try!(serde_json::from_reader(self)))
    }
}

impl std::io::Read for Response {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.hyper_response.read(buf)
    }
}

macro_rules! implement_method {
    ($name:ident, $method:expr, $doc:expr) => {
        #[doc = $doc]
        pub fn $name<U: IntoUrl>(url: U) -> Result<Request, url::ParseError> {
            Request::new($method, url)
        }
    }
}

implement_method!(options, Method::Options, "Create a OPTIONS request.");
implement_method!(get, Method::Get, "Create a GET request.");
implement_method!(post, Method::Post, "Create a POST request.");
implement_method!(put, Method::Put, "Create a PUT request.");
implement_method!(delete, Method::Delete, "Create a DELETE request.");
implement_method!(head, Method::Head, "Create a HEAD request.");
implement_method!(trace, Method::Trace, "Create a TRACE request.");
implement_method!(connect, Method::Connect, "Create a CONNECT request.");
implement_method!(patch, Method::Patch, "Create a PATCH request.");
