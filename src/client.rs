use reqwest::IntoUrl;
use serde_json;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, io::Read};

use super::consts::*;
use super::http_client::*;
use super::proxy::*;

pub struct Client {
    client: Arc<Mutex<HttpClient>>,
}

impl Client {
    pub fn new(toxiproxy_base_uri: impl IntoUrl) -> Self {
        Self {
            client: Arc::new(Mutex::new(HttpClient::new(toxiproxy_base_uri))),
        }
    }

    pub fn populate(&self, proxies: Vec<Proxy>) -> Result<Vec<Proxy>, String> {
        let proxies_json = serde_json::to_string(&proxies).unwrap();
        self.client
            .lock()
            .expect(ERR_LOCK)
            .post_with_data("populate", proxies_json)
            .and_then(|response| response.json::<HashMap<String, Vec<Proxy>>>())
            .map_err(|err| format!("<populate> has failed: {}", err))
            .map(|ref mut response_obj| response_obj.remove("proxies").unwrap_or(vec![]))
    }

    pub fn reset(&self) -> Result<(), String> {
        self.client
            .lock()
            .expect(ERR_LOCK)
            .post("reset")
            .map(|_| ())
            .map_err(|err| format!("<reset> has failed: {}", err))
    }

    pub fn all(&self) -> Result<HashMap<String, Proxy>, String> {
        self.client
            .lock()
            .expect(ERR_LOCK)
            .get("proxies")
            .and_then(|response| {
                response
                    .json()
                    .map(|mut proxy_map: HashMap<String, Proxy>| {
                        for proxy in proxy_map.values_mut() {
                            proxy.client = Some(self.client.clone());
                        }
                        proxy_map
                    })
            })
            .map_err(|err| format!("<proxies> has failed: {}", err))
    }

    pub fn is_running(&self) -> bool {
        self.client.lock().expect("Client lock failed").is_alive()
    }

    pub fn version(&self) -> Result<String, String> {
        self.client
            .lock()
            .expect(ERR_LOCK)
            .get("version")
            .map(|ref mut response| {
                let mut body = String::new();
                response
                    .read_to_string(&mut body)
                    .expect("HTTP response cannot be read");
                body
            })
            .map_err(|err| format!("<version> has failed: {}", err))
    }

    pub fn find_proxy(&self, name: &str) -> Result<Proxy, String> {
        let path = format!("proxies/{}", name);

        let proxy_result = self
            .client
            .lock()
            .expect(ERR_LOCK)
            .get(&path)
            .and_then(|response| response.json());

        proxy_result
            .map(|mut proxy: Proxy| {
                proxy.client = Some(self.client.clone());
                proxy
                    .delete_all_toxics()
                    .expect("proxy cannot reset toxics");
                proxy
            })
            .map_err(|err| format!("<proxies> has failed: {}", err))
    }
}
