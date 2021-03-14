use super::consts::*;
use super::http_client::*;
use super::toxic::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyPack {
    pub name: String,
    listen: String,
    upstream: String,
    pub enabled: bool,
    toxics: Vec<ToxicPack>,
}

impl ProxyPack {
    pub fn new(name: String, listen: String, upstream: String) -> Self {
        Self {
            name,
            listen,
            upstream,
            enabled: true,
            toxics: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Proxy {
    pub proxy_pack: ProxyPack,
    pub client: Arc<Mutex<HttpClient>>,
}

impl Proxy {
    pub fn new(proxy_pack: ProxyPack, client: Arc<Mutex<HttpClient>>) -> Self {
        Self { proxy_pack, client }
    }

    fn disable(&self) -> Result<(), String> {
        let mut payload: HashMap<String, bool> = HashMap::new();
        payload.insert("enabled".into(), false);
        let body = serde_json::to_string(&payload).map_err(|_| ERR_JSON_SERIALIZE)?;

        self.update(body)
    }

    fn enable(&self) -> Result<(), String> {
        let mut payload: HashMap<String, bool> = HashMap::new();
        payload.insert("enabled".into(), true);
        let body = serde_json::to_string(&payload).map_err(|_| ERR_JSON_SERIALIZE)?;

        self.update(body)
    }

    pub fn update(&self, payload: String) -> Result<(), String> {
        let path = format!("proxies/{}", self.proxy_pack.name);

        self.client
            .lock()
            .map_err(|err| format!("lock error: {}", err))?
            .post_with_data(&path, payload)
            .map_err(|err| format!("<disable> has failed: {}", err))
            .map(|_| ())
    }

    pub fn delete(&self) -> Result<(), String> {
        let path = format!("proxies/{}", self.proxy_pack.name);

        self.client
            .lock()
            .map_err(|err| format!("lock error: {}", err))?
            .delete(&path)
            .map_err(|err| format!("<disable> has failed: {}", err))
            .map(|_| ())
    }

    pub fn toxics(&self) -> Result<Vec<ToxicPack>, String> {
        let path = format!("proxies/{}/toxics", self.proxy_pack.name);

        self.client
            .lock()
            .map_err(|err| format!("lock error: {}", err))?
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

        self.create_toxic(ToxicPack::new(
            "latency".into(),
            stream,
            toxicity,
            attributes,
        ))
    }

    pub fn with_bandwidth(&self, stream: String, rate: ToxicValueType, toxicity: f32) -> &Self {
        let mut attributes = HashMap::new();
        attributes.insert("rate".into(), rate);

        self.create_toxic(ToxicPack::new(
            "bandwidth".into(),
            stream,
            toxicity,
            attributes,
        ))
    }

    pub fn with_slow_close(&self, stream: String, delay: ToxicValueType, toxicity: f32) -> &Self {
        let mut attributes = HashMap::new();
        attributes.insert("delay".into(), delay);

        self.create_toxic(ToxicPack::new(
            "slow_close".into(),
            stream,
            toxicity,
            attributes,
        ))
    }

    pub fn with_timeout(&self, stream: String, timeout: ToxicValueType, toxicity: f32) -> &Self {
        let mut attributes = HashMap::new();
        attributes.insert("timeout".into(), timeout);

        self.create_toxic(ToxicPack::new(
            "timeout".into(),
            stream,
            toxicity,
            attributes,
        ))
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

        self.create_toxic(ToxicPack::new(
            "slicer".into(),
            stream,
            toxicity,
            attributes,
        ))
    }

    pub fn with_limit_data(&self, stream: String, bytes: ToxicValueType, toxicity: f32) -> &Self {
        let mut attributes = HashMap::new();
        attributes.insert("bytes".into(), bytes);

        self.create_toxic(ToxicPack::new(
            "limit_data".into(),
            stream,
            toxicity,
            attributes,
        ))
    }

    fn create_toxic(&self, toxic: ToxicPack) -> &Self {
        let body = serde_json::to_string(&toxic).expect(ERR_JSON_SERIALIZE);
        let path = format!("proxies/{}/toxics", self.proxy_pack.name);

        let _ = self
            .client
            .lock()
            .expect(ERR_LOCK)
            .post_with_data(&path, body)
            .map_err(|err| {
                panic!("<proxies>.<toxics> creation has failed: {}", err);
            });

        self
    }

    pub fn down<F>(&self, closure: F) -> Result<(), String>
    where
        F: FnOnce(),
    {
        self.disable()?;
        closure();
        self.enable()
    }

    pub fn apply<F>(&self, closure: F) -> Result<(), String>
    where
        F: FnOnce(),
    {
        closure();
        self.delete_all_toxics()
    }

    pub fn delete_all_toxics(&self) -> Result<(), String> {
        self.toxics()
            .and_then(|toxic_list| {
                for toxic in toxic_list {
                    let path = format!("proxies/{}/toxics/{}", self.proxy_pack.name, toxic.name);
                    let result = self
                        .client
                        .lock()
                        .map_err(|err| format!("lock error: {}", err))?
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
