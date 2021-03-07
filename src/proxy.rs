use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::consts::*;
use super::http_client::*;
use super::toxic::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Proxy {
    pub name: String,
    listen: String,
    upstream: String,
    pub enabled: bool,
    toxics: Vec<Toxic>,

    #[serde(skip)]
    pub client: Option<Arc<Mutex<HttpClient>>>,
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

    pub fn toxics(&self) -> Result<Vec<Toxic>, String> {
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
                    let path = format!("/proxies/{}/toxics/{}", self.name, toxic.name);
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
