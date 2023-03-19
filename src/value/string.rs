use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct JsonString(String);

impl JsonString {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

impl Deref for JsonString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToString for JsonString {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
