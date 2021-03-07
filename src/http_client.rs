use http;
use reqwest::{self, blocking::Client};

#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    toxiproxy_base_uri: String,
}

impl HttpClient {
    pub(crate) fn new(toxiproxy_base_uri: String) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            toxiproxy_base_uri,
        }
    }

    pub(crate) fn get(&self, path: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
        self.client
            .get(&self.uri_with_path(path))
            .header("Content-Type", "application/json")
            .send()
    }

    pub(crate) fn post(&self, path: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
        self.client
            .post(&self.uri_with_path(path))
            .header("Content-Type", "application/json")
            .send()
    }

    pub(crate) fn post_with_data(
        &self,
        path: &str,
        body: String,
    ) -> Result<reqwest::blocking::Response, reqwest::Error> {
        self.client
            .post(&self.uri_with_path(path))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
    }

    pub(crate) fn delete(&self, path: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
        self.client
            .delete(&self.uri_with_path(path))
            .header("Content-Type", "application/json")
            .send()
    }

    fn uri_with_path(&self, path: &str) -> String {
        let mut full_uri = self.toxiproxy_base_uri.clone();
        full_uri.push_str(path);
        full_uri
    }

    pub(crate) fn is_alive(&self) -> bool {
        let addr = self
            .toxiproxy_base_uri
            .parse::<http::Uri>()
            .expect("Toxiproxy URI provided is not valid")
            .authority()
            .expect("Invalid authority component")
            .to_string();

        std::net::TcpStream::connect(addr)
            .map(|_| true)
            .unwrap_or(false)
    }
}
