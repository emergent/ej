#[derive(Debug, Clone, PartialEq)]
pub enum JsonNumber {
    Integer(i64),
    Float(f64),
}

impl ToString for JsonNumber {
    fn to_string(&self) -> String {
        match self {
            JsonNumber::Integer(i) => i.to_string(),
            JsonNumber::Float(f) => f.to_string(),
        }
    }
}
