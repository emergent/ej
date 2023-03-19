#[derive(Debug)]
pub struct JsonBool(bool);

impl JsonBool {
    pub fn new(b: bool) -> Self {
        Self(b)
    }
}

impl ToString for JsonBool {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
