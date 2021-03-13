use http;
use reqwest::{self, blocking::Client, IntoUrl, Url};

#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    toxiproxy_base_uri: Url,
}

impl HttpClient {
    pub(crate) fn new(toxiproxy_base_uri: impl IntoUrl) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            toxiproxy_base_uri: toxiproxy_base_uri.into_url().expect("Incorrect URL format"),
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
        let mut full_uri: String = self.toxiproxy_base_uri.as_str().into();
        full_uri.push_str(path);
        full_uri
    }

    pub(crate) fn is_alive(&self) -> bool {
        let addr = self
            .toxiproxy_base_uri
            .as_str()
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
