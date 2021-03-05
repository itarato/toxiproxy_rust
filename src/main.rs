/**

x   GET /proxies - List existing proxies and their toxics
    POST /proxies - Create a new proxy
x   POST /populate - Create or replace a list of proxies
    GET /proxies/{proxy} - Show the proxy with all its active toxics
    POST /proxies/{proxy} - Update a proxy's fields
    DELETE /proxies/{proxy} - Delete an existing proxy
    GET /proxies/{proxy}/toxics - List active toxics
    POST /proxies/{proxy}/toxics - Create a new toxic
    GET /proxies/{proxy}/toxics/{toxic} - Get an active toxic's fields
    POST /proxies/{proxy}/toxics/{toxic} - Update an active toxic
    DELETE /proxies/{proxy}/toxics/{toxic} - Remove an active toxic
x   POST /reset - Enable all proxies and remove all active toxics
x   GET /version - Returns the server version number

**/

#[macro_use]
extern crate lazy_static;

use http;
use reqwest::{self, blocking::Client};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::HashMap, io::Read};

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
    attributes: HashMap<String, String>,
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

    pub fn version(&self) -> Result<String, String> {
        self.get("/version")
            .map(|ref mut response| {
                let mut body = String::new();
                response
                    .read_to_string(&mut body)
                    .expect("HTTP response cannot be read");
                body
            })
            .map_err(|err| format!("<version> has failed: {}", err))
    }

    pub fn find_proxy(&self, name: &str) -> Option<Proxy> {
        self.all()
            .map(|ref mut proxy_map| proxy_map.remove(name))
            .unwrap_or(None)
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
    dbg!(TOXIPROXY.find_proxy("socket"));
    dbg!(TOXIPROXY.version());
}
