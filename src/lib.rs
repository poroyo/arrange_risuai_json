use anyhow::{Error, anyhow};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use web_sys::console;

pub fn console_log(msg: &str) {
    console::log_1(&format!("{:?}", msg).into());
}

pub fn console_error(msg: &str) {
    console::error_1(&format!("{:?}", msg).into())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum JsonType {
    #[serde(rename = "risu")]
    LoreBook(LorebookJson),
    #[serde(rename = "regex")]
    Regex(RegexJson),
    Nil,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LorebookJson {
    #[serde(rename = "type")]
    pub app_type: String,
    pub ver: u8,
    pub data: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegexJson {
    #[serde(rename = "type")]
    pub app_type: String,
    pub data: Vec<serde_json::Value>,
}

#[derive(Debug)]
pub struct Data {
    pub data: Vec<serde_json::Value>,
    pub is_lore: bool,
    pub lore_ver: Option<u8>,
}

pub struct HasherbleValue(pub serde_json::Value, pub usize);

impl Hash for HasherbleValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_value(&self.0, state);
    }
}

fn hash_value<H: Hasher>(value: &serde_json::Value, state: &mut H) {
    use serde_json::Value;

    match value {
        Value::Null => 0.hash(state),
        Value::Number(n) => n.hash(state),
        Value::String(s) => s.hash(state),
        Value::Bool(b) => b.hash(state),
        Value::Array(arr) => arr.iter().for_each(|a| hash_value(a, state)),
        Value::Object(obj) => obj.iter().for_each(|e| {
            e.0.hash(state);
            hash_value(e.1, state);
        }),
    }
}

impl Default for JsonType {
    fn default() -> Self {
        Self::Nil
    }
}

pub fn file_read(file_contents: Vec<u8>) -> Result<Data, Error> {
    let contents = String::from_utf8(file_contents)?;
    let contents: JsonType = serde_json::from_str(&contents)?;

    match contents {
        JsonType::LoreBook(l) => {
            Ok(Data {
                data: l.data,
                is_lore: true,
                lore_ver: Some(l.ver),
            })
        }
        JsonType::Regex(r) => {
            Ok(Data { data: r.data, is_lore: false, lore_ver: None })

        }
        JsonType::Nil => Err(anyhow!("Invalid file")),
    }
}


// #[allow(non_snake_case)]
// #[derive(Debug, Deserialize, Serialize)]
// struct Data {
//     key: String,
//     comment: String,
//     content: String,
//     mode: String,
//     insertorder: u32,
//     alwaysActive: bool,
//     secondkey: String,
//     selective: bool,
//     activationPercent: Option<serde_json::Value>,
// }
