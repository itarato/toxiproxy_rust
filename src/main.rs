#[macro_use]
extern crate lazy_static;

use http;
use reqwest::{self, blocking::Client};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

const TOXIPROXY_DEFAULT_URI: &str = "http://127.0.0.1:8474";
lazy_static! {
    static ref TOXIPROXY: Toxiproxy = Toxiproxy::new(TOXIPROXY_DEFAULT_URI.into());
}

#[derive(Serialize, Deserialize, Debug)]
struct Toxic {
    name: String,
    r#type: String,
    stream: String,
    toxicity: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Proxy {
    name: String,
    listen: String,
    upstream: String,
    enabled: bool,
    toxics: Vec<Toxic>,
}

impl Proxy {
    fn new(name: String, listen: String, upstream: String) -> Self {
        Self {
            name,
            listen,
            upstream,
            enabled: true,
            toxics: vec![],
        }
    }
}

struct Toxiproxy {
    client: Client,
    toxiproxy_base_uri: String,
}

impl Toxiproxy {
    fn new(toxiproxy_base_uri: String) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            toxiproxy_base_uri,
        }
    }

    pub fn populate(&self, proxies: Vec<Proxy>) -> Result<Vec<Proxy>, String> {
        let proxies_json = serde_json::to_string(&proxies).unwrap();
        self.post_with_data("/populate", proxies_json)
            .and_then(|response| response.json::<HashMap<String, Vec<Proxy>>>())
            .map_err(|err| format!("<populate> has failed: {}", err))
            .map(|ref mut response_obj| response_obj.remove("proxies").unwrap_or(vec![]))
    }

    pub fn reset(&self) -> Result<(), String> {
        self.post("/reset")
            .map(|_| ())
            .map_err(|err| format!("<reset> has failed: {}", err))
    }

    pub fn all(&self) -> Result<HashMap<String, Proxy>, String> {
        self.get("/proxies")
            .and_then(|response| response.json())
            .map_err(|err| format!("<proxies> has failed: {}", err))
    }

    pub fn is_running(&self) -> bool {
        let uri = self
            .toxiproxy_base_uri
            .parse::<http::Uri>()
            .expect("Toxiproxy URI provided is not valid");

        std::net::TcpStream::connect(uri.authority().expect("Invalid URI").to_string())
            .map(|_| true)
            .unwrap_or(false)
    }

    fn get(&self, path: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
        self.client
            .get(&self.uri_with_path(path))
            .header("Content-Type", "application/json")
            .send()
    }

    fn post(&self, path: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
        self.client
            .post(&self.uri_with_path(path))
            .header("Content-Type", "application/json")
            .send()
    }

    fn post_with_data(
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

    fn uri_with_path(&self, path: &str) -> String {
        let mut full_uri = self.toxiproxy_base_uri.clone();
        full_uri.push_str(path);
        full_uri
    }
}

fn main() {
    dbg!(TOXIPROXY.is_running());
    dbg!(TOXIPROXY.reset());
    dbg!(TOXIPROXY.populate(vec![Proxy::new(
        "socket".into(),
        "127.0.0.1:2000".into(),
        "127.0.0.1:2001".into(),
    )]));
    dbg!(TOXIPROXY.all());
}
