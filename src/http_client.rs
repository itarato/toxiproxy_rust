use reqwest::{blocking::Client, blocking::Response, Url};
use std::{
    net::{SocketAddr, ToSocketAddrs},
    str::FromStr,
};

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

    pub(crate) fn get(&self, path: &str) -> Result<Response, String> {
        self.client
            .get(self.uri_with_path(path)?)
            .header("Content-Type", "application/json")
            .send()
            .map_err(|err| format!("GET error: {}", err))
    }

    pub(crate) fn post(&self, path: &str) -> Result<Response, String> {
        self.client
            .post(self.uri_with_path(path)?)
            .header("Content-Type", "application/json")
            .send()
            .map_err(|err| format!("POST error: {}", err))
    }

    pub(crate) fn post_with_data(&self, path: &str, body: String) -> Result<Response, String> {
        self.client
            .post(self.uri_with_path(path)?)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .map_err(|err| format!("POST error: {}", err))
    }

    pub(crate) fn delete(&self, path: &str) -> Result<Response, String> {
        self.client
            .delete(self.uri_with_path(path)?)
            .header("Content-Type", "application/json")
            .send()
            .map_err(|err| format!("DELETE error: {}", err))
    }

    fn uri_with_path(&self, path: &str) -> Result<Url, String> {
        let mut base: String = "http://".into();
        base.push_str(&self.toxiproxy_addr.to_string());

        let mut url = Url::from_str(&base).map_err(|err| format!("Incorrect address: {}", err))?;

        url.set_scheme("http")
            .map_err(|_| "invalid scheme".to_owned())?;
        url.set_path(path);
        Ok(url)
    }

    pub(crate) fn is_alive(&self) -> bool {
        std::net::TcpStream::connect(self.toxiproxy_addr)
            .map(|_| true)
            .unwrap_or(false)
    }
}
