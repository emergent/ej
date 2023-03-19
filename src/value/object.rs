use super::{string::JsonString, JsonValue};
use std::collections::HashMap;

#[derive(Debug)]
pub struct JsonObject(HashMap<JsonString, JsonValue>);

impl JsonObject {
    pub fn new(map: HashMap<JsonString, JsonValue>) -> Self {
        Self(map)
    }
}

impl<'a> IntoIterator for &'a JsonObject {
    type Item = (&'a JsonString, &'a JsonValue);
    type IntoIter = JsonObjectIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        JsonObjectIter(self.0.iter())
    }
}

pub struct JsonObjectIter<'a>(std::collections::hash_map::Iter<'a, JsonString, JsonValue>);

impl<'a> Iterator for JsonObjectIter<'a> {
    type Item = (&'a JsonString, &'a JsonValue);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
