//! Main client for communicating with the Toxiproxy server.

use serde_json;
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, io::Read};

use super::http_client::*;
use super::proxy::*;

#[derive(Clone)]
pub struct Client {
    client: Arc<Mutex<HttpClient>>,
}

impl Client {
    pub fn new<U: ToSocketAddrs>(toxiproxy_addr: U) -> Self {
        Self {
            client: Arc::new(Mutex::new(HttpClient::new(toxiproxy_addr))),
        }
    }

    pub fn populate(&self, proxies: Vec<ProxyPack>) -> Result<Vec<Proxy>, String> {
        let proxies_json = serde_json::to_string(&proxies).unwrap();
        self.client
            .lock()
            .map_err(|err| format!("lock error: {}", err))?
            .post_with_data("populate", proxies_json)
            .and_then(|response| {
                response
                    .json::<HashMap<String, Vec<ProxyPack>>>()
                    .map_err(|err| format!("json deserialize failed: {}", err))
            })
            .map(|ref mut response_obj| response_obj.remove("proxies").unwrap_or(vec![]))
            .map(|proxy_packs| {
                proxy_packs
                    .into_iter()
                    .map(|proxy_pack| Proxy::new(proxy_pack, self.client.clone()))
                    .collect::<Vec<Proxy>>()
            })
    }

    pub fn reset(&self) -> Result<(), String> {
        self.client
            .lock()
            .map_err(|err| format!("lock error: {}", err))?
            .post("reset")
            .map(|_| ())
            .map_err(|err| format!("<reset> has failed: {}", err))
    }

    pub fn all(&self) -> Result<HashMap<String, Proxy>, String> {
        self.client
            .lock()
            .map_err(|err| format!("lock error: {}", err))?
            .get("proxies")
            .and_then(|response| {
                response
                    .json()
                    .map(|proxy_map: HashMap<String, ProxyPack>| {
                        proxy_map
                            .into_iter()
                            .map(|(name, proxy_pack)| {
                                (name, Proxy::new(proxy_pack, self.client.clone()))
                            })
                            .collect()
                    })
                    .map_err(|err| format!("json deserialize failed: {}", err))
            })
    }

    pub fn is_running(&self) -> bool {
        self.client.lock().expect("Client lock failed").is_alive()
    }

    pub fn version(&self) -> Result<String, String> {
        self.client
            .lock()
            .map_err(|err| format!("lock error: {}", err))?
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
            .map_err(|err| format!("lock error: {}", err))?
            .get(&path)
            .and_then(|response| {
                response
                    .json()
                    .map_err(|err| format!("json deserialize failed: {}", err))
            });

        proxy_result
            .map(|proxy_pack: ProxyPack| {
                let proxy = Proxy::new(proxy_pack, self.client.clone());
                proxy
                    .delete_all_toxics()
                    .expect("proxy cannot reset toxics");
                proxy
            })
            .map_err(|err| format!("<proxies> has failed: {}", err))
    }
}
