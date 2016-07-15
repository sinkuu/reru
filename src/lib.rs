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

pub struct RestRequest {
    pub method: Method,
    pub url: Url,
    pub headers: Headers,
    pub body: Body,
    client: Option<Client>,
}

impl RestRequest {
    pub fn new<U: IntoUrl>(method: Method, url: U) -> Result<RestRequest, url::ParseError> {
        Ok(RestRequest {
            method: method,
            url: try!(url.into_url()),
            headers: Headers::new(),
            // form: None,
            body: Body::None,
            client: None,
        })
    }

    pub fn param(mut self, name: &str, value: &str) -> Self {
        self.url.query_pairs_mut().append_pair(name, value);
        self
    }

    #[cfg(feature = "json")]
    pub fn body_json<T: Serialize>(mut self, value: &T) -> Result<Self, serde_json::error::Error> {
        self.body = Body::Buffer(try!(serde_json::to_vec(value)));
        self.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
        Ok(self)
    }

    pub fn body_form(mut self, name: &str, value: &str) -> Self {
        match self.body {
            Body::None | Body::Buffer(_) => {
                self.body = Body::Forms(Serializer::new(String::new()));
                self.headers
                    .set(ContentType(Mime(TopLevel::Application,
                                          SubLevel::WwwFormUrlEncoded,
                                          vec![])));
            }

            Body::Forms(_) => (),
        }

        if let Body::Forms(ref mut ser) = self.body {
            ser.append_pair(name, value);
        }

        self
    }

    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);

        self
    }

    pub fn request(self) -> HyperResult<Response> {
        let client = self.client.unwrap_or_else(|| Client::new());

        // let c = client.request(..) <-- This outlives `encoded`

        Ok(Response::new(try!(match self.body {
            Body::Buffer(ref body) => {
                client.request(self.method, self.url)
                    .headers(self.headers)
                    .body(hyper::client::Body::BufBody(&body, body.len()))
                    .send()
            }

            Body::Forms(mut ser) => {
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

pub enum Body {
    None,
    Buffer(Vec<u8>),
    Forms(Serializer<String>),
}

pub struct Response {
    hyper_response: HyperResponse,
}

impl Response {
    #[inline]
    fn new(hyper_response: HyperResponse) -> Response {
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
    ($name:ident, $method:expr) => {
        pub fn $name<U: IntoUrl>(url: U) -> Result<RestRequest, url::ParseError> {
            RestRequest::new($method, url)
        }
    }
}

implement_method!(options, Method::Options);
implement_method!(get, Method::Get);
implement_method!(post, Method::Post);
implement_method!(put, Method::Put);
implement_method!(delete, Method::Delete);
implement_method!(head, Method::Head);
implement_method!(trace, Method::Trace);
implement_method!(connect, Method::Connect);
implement_method!(patch, Method::Patch);
