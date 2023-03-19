use std::ops::Deref;

use super::JsonValue;

#[derive(Debug)]
pub struct JsonArray(Vec<JsonValue>);

impl JsonArray {
    pub fn new(array: Vec<JsonValue>) -> Self {
        Self(array)
    }
}

impl Deref for JsonArray {
    type Target = Vec<JsonValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for JsonArray {
    type Item = JsonValue;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a JsonArray {
    type Item = &'a JsonValue;
    type IntoIter = JsonArrayIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        JsonArrayIter(self.0.iter())
    }
}

pub struct JsonArrayIter<'a>(std::slice::Iter<'a, JsonValue>);

impl<'a> Iterator for JsonArrayIter<'a> {
    type Item = &'a JsonValue;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
