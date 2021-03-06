#[macro_use]
extern crate lazy_static;

use http;
use reqwest::{self, blocking::Client};
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, io::Read};

type ToxicValueType = u32;

const TOXIPROXY_DEFAULT_URI: &str = "http://127.0.0.1:8474";
const ERR_MISSING_HTTP_CLIENT: &str = "HTTP client not available";
const ERR_LOCK: &str = "Lock cannot be granted";
const ERR_JSON_SERIALIZE: &str = "JSON serialization failed";

lazy_static! {
    pub static ref TOXIPROXY: Toxiproxy = Toxiproxy::new(TOXIPROXY_DEFAULT_URI.into());
}

#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    toxiproxy_base_uri: String,
}

impl HttpClient {
    fn new(toxiproxy_base_uri: String) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            toxiproxy_base_uri,
        }
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

    fn delete(&self, path: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
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

    fn is_alive(&self) -> bool {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Toxic {
    name: String,
    r#type: String,
    stream: String,
    toxicity: f32,
    attributes: HashMap<String, ToxicValueType>,
}

impl Toxic {
    fn new(
        r#type: String,
        stream: String,
        toxicity: f32,
        attributes: HashMap<String, ToxicValueType>,
    ) -> Self {
        let name = format!("{}_{}", r#type, stream);
        Self {
            name,
            r#type,
            stream,
            toxicity,
            attributes,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Proxy {
    name: String,
    listen: String,
    upstream: String,
    enabled: bool,
    toxics: Vec<Toxic>,

    #[serde(skip)]
    client: Option<Arc<Mutex<HttpClient>>>,
}

impl Proxy {
    pub fn new(name: String, listen: String, upstream: String) -> Self {
        Self {
            name,
            listen,
            upstream,
            enabled: true,
            toxics: vec![],
            client: None,
        }
    }

    fn with_client(mut self, client: Arc<Mutex<HttpClient>>) -> Self {
        self.client = Some(client);
        self
    }

    fn disable(&self) -> Result<(), String> {
        let mut payload: HashMap<String, bool> = HashMap::new();
        payload.insert("enabled".into(), false);
        let body = serde_json::to_string(&payload).expect("Failed serializing");

        self.update(body)
    }

    fn enable(&self) -> Result<(), String> {
        let mut payload: HashMap<String, bool> = HashMap::new();
        payload.insert("enabled".into(), true);
        let body = serde_json::to_string(&payload).expect("Failed serializing");

        self.update(body)
    }

    pub fn update(&self, payload: String) -> Result<(), String> {
        let path = format!("/proxies/{}", self.name);

        self.client
            .as_ref()
            .expect(ERR_MISSING_HTTP_CLIENT)
            .lock()
            .expect(ERR_LOCK)
            .post_with_data(&path, payload)
            .map_err(|err| format!("<disable> has failed: {}", err))
            .map(|_| ())
    }

    pub fn delete(&self) -> Result<(), String> {
        let path = format!("/proxies/{}", self.name);

        self.client
            .as_ref()
            .expect(ERR_MISSING_HTTP_CLIENT)
            .lock()
            .expect(ERR_LOCK)
            .delete(&path)
            .map_err(|err| format!("<disable> has failed: {}", err))
            .map(|_| ())
    }

    fn toxics(&self) -> Result<Vec<Toxic>, String> {
        let path = format!("/proxies/{}/toxics", self.name);

        self.client
            .as_ref()
            .expect(ERR_MISSING_HTTP_CLIENT)
            .lock()
            .expect(ERR_LOCK)
            .get(&path)
            .and_then(|response| response.json())
            .map_err(|err| format!("<proxies>.<toxics> has failed: {}", err))
    }

    pub fn with_latency(
        &self,
        stream: String,
        latency: ToxicValueType,
        jitter: ToxicValueType,
        toxicity: f32,
    ) -> &Self {
        let mut attributes = HashMap::new();
        attributes.insert("latency".into(), latency);
        attributes.insert("jitter".into(), jitter);

        self.create_toxic(Toxic::new("latency".into(), stream, toxicity, attributes))
    }

    pub fn with_bandwidth(&self, stream: String, rate: ToxicValueType, toxicity: f32) -> &Self {
        let mut attributes = HashMap::new();
        attributes.insert("rate".into(), rate);

        self.create_toxic(Toxic::new("bandwidth".into(), stream, toxicity, attributes))
    }

    pub fn with_slow_close(&self, stream: String, delay: ToxicValueType, toxicity: f32) -> &Self {
        let mut attributes = HashMap::new();
        attributes.insert("delay".into(), delay);

        self.create_toxic(Toxic::new(
            "slow_close".into(),
            stream,
            toxicity,
            attributes,
        ))
    }

    pub fn with_timeout(&self, stream: String, timeout: ToxicValueType, toxicity: f32) -> &Self {
        let mut attributes = HashMap::new();
        attributes.insert("timeout".into(), timeout);

        self.create_toxic(Toxic::new("timeout".into(), stream, toxicity, attributes))
    }

    pub fn with_slicer(
        &self,
        stream: String,
        average_size: ToxicValueType,
        size_variation: ToxicValueType,
        delay: ToxicValueType,
        toxicity: f32,
    ) -> &Self {
        let mut attributes = HashMap::new();
        attributes.insert("average_size".into(), average_size);
        attributes.insert("size_variation".into(), size_variation);
        attributes.insert("delay".into(), delay);

        self.create_toxic(Toxic::new("slicer".into(), stream, toxicity, attributes))
    }

    pub fn with_limit_data(&self, stream: String, bytes: ToxicValueType, toxicity: f32) -> &Self {
        let mut attributes = HashMap::new();
        attributes.insert("bytes".into(), bytes);

        self.create_toxic(Toxic::new(
            "limit_data".into(),
            stream,
            toxicity,
            attributes,
        ))
    }

    fn create_toxic(&self, toxic: Toxic) -> &Self {
        let body = serde_json::to_string(&toxic).expect(ERR_JSON_SERIALIZE);
        let path = format!("/proxies/{}/toxics", self.name);

        let _ = self
            .client
            .as_ref()
            .expect(ERR_MISSING_HTTP_CLIENT)
            .lock()
            .expect(ERR_LOCK)
            .post_with_data(&path, body)
            .map_err(|err| {
                panic!("<proxies>.<toxics> creation has failed: {}", err);
            });

        self
    }

    pub fn down<F>(&self, closure: F)
    where
        F: FnOnce(),
    {
        self.disable().expect("cannot be disabled");
        closure();
        self.enable().expect("cannot be enabled");
    }

    pub fn apply<F>(&self, closure: F) -> Result<(), String>
    where
        F: FnOnce(),
    {
        closure();
        self.delete_all_toxics()
    }

    fn delete_all_toxics(&self) -> Result<(), String> {
        self.toxics()
            .and_then(|toxic_list| {
                for toxic in toxic_list {
                    let path = format!("/proxies/{}/toxics.{}", self.name, toxic.name);
                    let result = self
                        .client
                        .as_ref()
                        .expect(ERR_MISSING_HTTP_CLIENT)
                        .lock()
                        .expect(ERR_LOCK)
                        .delete(&path);

                    if result.is_err() {
                        return Err(format!(
                            "<proxies>.<toxics> delete failed: {}",
                            result.err().unwrap()
                        ));
                    }
                }

                Ok(())
            })
            .map_err(|err| format!("cannot delete toxics: {}", err))
    }
}

pub struct Toxiproxy {
    client: Arc<Mutex<HttpClient>>,
}

impl Toxiproxy {
    fn new(toxiproxy_base_uri: String) -> Self {
        Self {
            client: Arc::new(Mutex::new(HttpClient::new(toxiproxy_base_uri))),
        }
    }

    pub fn populate(&self, proxies: Vec<Proxy>) -> Result<Vec<Proxy>, String> {
        let proxies_json = serde_json::to_string(&proxies).unwrap();
        self.client
            .lock()
            .expect("Client lock failed")
            .post_with_data("/populate", proxies_json)
            .and_then(|response| response.json::<HashMap<String, Vec<Proxy>>>())
            .map_err(|err| format!("<populate> has failed: {}", err))
            .map(|ref mut response_obj| response_obj.remove("proxies").unwrap_or(vec![]))
    }

    pub fn reset(&self) -> Result<(), String> {
        self.client
            .lock()
            .expect("Client lock failed")
            .post("/reset")
            .map(|_| ())
            .map_err(|err| format!("<reset> has failed: {}", err))
    }

    pub fn all(&self) -> Result<HashMap<String, Proxy>, String> {
        self.client
            .lock()
            .expect("Client lock failed")
            .get("/proxies")
            .and_then(|response| response.json())
            .map_err(|err| format!("<proxies> has failed: {}", err))
    }

    pub fn is_running(&self) -> bool {
        self.client.lock().expect("Client lock failed").is_alive()
    }

    pub fn version(&self) -> Result<String, String> {
        self.client
            .lock()
            .expect("Client lock failed")
            .get("/version")
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
            .map(|ref mut proxy_map| {
                proxy_map.remove(name).map(|proxy| {
                    let proxy = proxy.with_client(self.client.clone());
                    proxy
                        .delete_all_toxics()
                        .expect("proxy cannot reset toxics");
                    proxy
                })
            })
            .unwrap_or(None)
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        // dbg!(TOXIPROXY.is_running());
        dbg!(TOXIPROXY.reset());
        dbg!(TOXIPROXY.populate(vec![Proxy::new(
            "socket".into(),
            "localhost:2001".into(),
            "localhost:2000".into(),
        )]));
        // dbg!(TOXIPROXY.all());
        // dbg!(TOXIPROXY.version());

        let proxy = dbg!(TOXIPROXY.find_proxy("socket").unwrap());
        // dbg!(proxy.disable());
        // dbg!(proxy.enable());

        proxy
            .with_latency("downstream".into(), 2000, 0, 1.0)
            .apply(|| {
                use std::io::prelude::*;
                use std::net::TcpStream;
                use std::time::SystemTime;

                println!("START {:?}", SystemTime::now());

                dbg!(TOXIPROXY.all());

                let mut stream =
                    TcpStream::connect("localhost:2001").expect("stream cannot be created");

                let mut out = String::new();

                stream.read_to_string(&mut out).expect("read body failed");

                // dbg!(out);
                println!("END {:?}", SystemTime::now());
            });
        dbg!(TOXIPROXY.all());
    }
}
