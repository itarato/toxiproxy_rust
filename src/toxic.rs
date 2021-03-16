//! Represents a [Toxic] - an effect on the network connection.
//!
//! [Toxic]: https://github.com/Shopify/toxiproxy#toxics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ToxicValueType = u32;

/// Config of a Toxic.
#[derive(Serialize, Deserialize, Debug)]
pub struct ToxicPack {
    pub name: String,
    pub r#type: String,
    pub stream: String,
    pub toxicity: f32,
    pub attributes: HashMap<String, ToxicValueType>,
}

impl ToxicPack {
    pub(crate) fn new(
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
