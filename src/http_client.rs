use http;
use reqwest::{blocking::Client, blocking::Response, Error, Url};
use std::net::{SocketAddr, ToSocketAddrs};

#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    toxiproxy_addr: SocketAddr,
}

impl HttpClient {
    pub(crate) fn new<U: ToSocketAddrs>(toxiproxy_addr: U) -> Self {
        Self {
            client: Client::new(),
            toxiproxy_addr: toxiproxy_addr.to_socket_addrs().unwrap().next().unwrap(),
        }
    }

    pub(crate) fn get(&self, path: &str) -> Result<Response, Error> {
        self.client
            .get(&self.uri_with_path(path))
            .header("Content-Type", "application/json")
            .send()
    }

    pub(crate) fn post(&self, path: &str) -> Result<Response, Error> {
        self.client
            .post(&self.uri_with_path(path))
            .header("Content-Type", "application/json")
            .send()
    }

    pub(crate) fn post_with_data(
        &self,
        path: &str,
        body: String,
    ) -> Result<reqwest::blocking::Response, Error> {
        self.client
            .post(&self.uri_with_path(path))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
    }

    pub(crate) fn delete(&self, path: &str) -> Result<Response, Error> {
        self.client
            .delete(&self.uri_with_path(path))
            .header("Content-Type", "application/json")
            .send()
    }

    fn uri_with_path(&self, path: &str) -> String {
        let mut full_uri: String = "http://".into();
        full_uri.push_str(&self.toxiproxy_addr.to_string());
        full_uri.push('/');
        full_uri.push_str(path);
        full_uri
    }

    pub(crate) fn is_alive(&self) -> bool {
        std::net::TcpStream::connect(self.toxiproxy_addr)
            .map(|_| true)
            .unwrap_or(false)
    }
}
