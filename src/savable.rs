use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Result;

pub trait Savable<'de, T: Serialize + Deserialize<'de>>
where
    Self: Serialize,
{
    fn from_json(serialized: &'de str) -> Result<T> {
        serde_json::from_str(serialized)
    }

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}
