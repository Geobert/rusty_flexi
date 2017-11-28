use serde_json;
use serde::{Serialize, Deserialize};

pub trait Savable<'de, T: Serialize + Deserialize<'de>>
where
    Self: Serialize,
{
    fn from_json(serialized: &'de str) -> T {
        serde_json::from_str(serialized).unwrap()
    }

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}
