//! Smart HTTP transport over reqwest (rustls).
//!
//! Implements git's Smart HTTP protocol (v0/v1) as a libgit2 subtransport.
//! Each `Service` maps to one HTTP request:
//!   UploadPackLs    GET  {url}/info/refs?service=git-upload-pack
//!   UploadPack      POST {url}/git-upload-pack
//!   ReceivePackLs   GET  {url}/info/refs?service=git-receive-pack
//!   ReceivePack     POST {url}/git-receive-pack
//!
//! libgit2 owns pkt-line framing and pack negotiation; we just shuttle bytes.

use std::io::{self, Read, Write};
use std::sync::Mutex;

use git2::transport::{Service, SmartSubtransport, SmartSubtransportStream, Transport};
use git2::{Error, Remote};
use reqwest::blocking::{Client, Response};
use reqwest::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};

const UA: &str = concat!("torii/", env!("CARGO_PKG_VERSION"));

pub fn factory(remote: &Remote<'_>) -> Result<Transport, Error> {
    Transport::smart(remote, true, HttpsSubtransport::new())
}

struct HttpsSubtransport {
    client: Client,
}

impl HttpsSubtransport {
    fn new() -> Self {
        let client = Client::builder()
            .user_agent(UA)
            .build()
            .expect("reqwest client");
        Self { client }
    }
}

impl SmartSubtransport for HttpsSubtransport {
    fn action(
        &self,
        url: &str,
        action: Service,
    ) -> Result<Box<dyn SmartSubtransportStream>, Error> {
        let stream = match action {
            Service::UploadPackLs => HttpStream::ls(&self.client, url, "git-upload-pack")?,
            Service::ReceivePackLs => HttpStream::ls(&self.client, url, "git-receive-pack")?,
            Service::UploadPack => HttpStream::rpc(self.client.clone(), url, "git-upload-pack"),
            Service::ReceivePack => HttpStream::rpc(self.client.clone(), url, "git-receive-pack"),
        };
        Ok(Box::new(stream))
    }

    fn close(&self) -> Result<(), Error> {
        Ok(())
    }
}

/// Bidirectional stream bridging libgit2's Read/Write to one HTTP request.
///
/// Two modes:
/// - `Ls`: GET completed at construction; reads stream the response body.
/// - `Rpc`: POST deferred. Writes buffer the request body. First read flushes
///   the buffer as the POST and then streams the response.
struct HttpStream {
    inner: Mutex<Inner>,
}

enum Inner {
    /// info/refs response — already fetched, just stream out.
    Ls { resp: Response },
    /// upload-pack / receive-pack — buffer writes, fire on first read.
    Rpc {
        client: Client,
        url: String,
        service: &'static str,
        sent: bool,
        req_body: Vec<u8>,
        resp: Option<Response>,
    },
}

impl HttpStream {
    fn ls(client: &Client, base_url: &str, service: &str) -> Result<Self, Error> {
        let url = format!("{}/info/refs?service={}", base_url.trim_end_matches('/'), service);
        let resp = client
            .get(&url)
            .header(USER_AGENT, UA)
            .header(ACCEPT, "*/*")
            .send()
            .map_err(io_err)?
            .error_for_status()
            .map_err(io_err)?;
        Ok(Self {
            inner: Mutex::new(Inner::Ls { resp }),
        })
    }

    fn rpc(client: Client, base_url: &str, service: &'static str) -> Self {
        let url = format!("{}/{}", base_url.trim_end_matches('/'), service);
        Self {
            inner: Mutex::new(Inner::Rpc {
                client,
                url,
                service,
                sent: false,
                req_body: Vec::new(),
                resp: None,
            }),
        }
    }
}

impl Read for HttpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut inner = self.inner.lock().unwrap();
        match &mut *inner {
            Inner::Ls { resp } => resp.read(buf),
            Inner::Rpc {
                client,
                url,
                service,
                sent,
                req_body,
                resp,
            } => {
                if !*sent {
                    let body = std::mem::take(req_body);
                    let req_ct = format!("application/x-{}-request", service);
                    let resp_ct = format!("application/x-{}-result", service);
                    let r = client
                        .post(url.as_str())
                        .header(USER_AGENT, UA)
                        .header(CONTENT_TYPE, req_ct)
                        .header(ACCEPT, resp_ct)
                        .body(body)
                        .send()
                        .map_err(to_io)?
                        .error_for_status()
                        .map_err(to_io)?;
                    *resp = Some(r);
                    *sent = true;
                }
                resp.as_mut().unwrap().read(buf)
            }
        }
    }
}

impl Write for HttpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut inner = self.inner.lock().unwrap();
        match &mut *inner {
            Inner::Ls { .. } => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "cannot write to Ls stream",
            )),
            Inner::Rpc { req_body, sent, .. } => {
                if *sent {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "cannot write after read started",
                    ));
                }
                req_body.extend_from_slice(buf);
                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn io_err(e: reqwest::Error) -> Error {
    Error::from_str(&format!("https transport: {}", e))
}

fn to_io(e: reqwest::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}
