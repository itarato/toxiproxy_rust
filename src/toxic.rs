use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ToxicValueType = u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct Toxic {
    pub name: String,
    r#type: String,
    stream: String,
    toxicity: f32,
    attributes: HashMap<String, ToxicValueType>,
}

impl Toxic {
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
