pub mod array;
pub mod bool;
pub mod null;
pub mod number;
pub mod object;
pub mod string;

use self::{
    array::JsonArray, bool::JsonBool, number::JsonNumber, object::JsonObject, string::JsonString,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Location(pub usize, pub usize);

impl Location {
    pub fn start(&self) -> usize {
        self.0
    }

    pub fn end(&self) -> usize {
        self.1
    }
}

#[derive(Debug)]
pub struct Annotation<T> {
    value: T,
    loc: Location,
}

impl<T> Annotation<T> {
    pub fn new(value: T, loc: Location) -> Self {
        Self { value, loc }
    }

    pub fn loc(&self) -> &Location {
        &self.loc
    }
}

#[derive(Debug)]
pub enum ValueKind {
    Object(JsonObject),
    Array(JsonArray),
    String(JsonString),
    Number(JsonNumber),
    Bool(JsonBool),
    Null,
}

pub type JsonValue = Annotation<ValueKind>;

impl JsonValue {
    pub fn kind(&self) -> &ValueKind {
        &self.value
    }

    pub fn into_kind(self) -> ValueKind {
        self.value
    }

    pub fn null(loc: Location) -> JsonValue {
        Self::new(ValueKind::Null, loc)
    }

    pub fn bool(value: bool, loc: Location) -> JsonValue {
        Self::new(ValueKind::Bool(JsonBool::new(value)), loc)
    }

    pub fn number_int(value: i64, loc: Location) -> JsonValue {
        Self::new(ValueKind::Number(JsonNumber::Integer(value)), loc)
    }

    pub fn number_float(value: f64, loc: Location) -> JsonValue {
        Self::new(ValueKind::Number(JsonNumber::Float(value)), loc)
    }

    pub fn string(value: String, loc: Location) -> JsonValue {
        Self::new(ValueKind::String(JsonString::new(value)), loc)
    }

    pub fn array(value: Vec<JsonValue>, loc: Location) -> JsonValue {
        Self::new(ValueKind::Array(JsonArray::new(value)), loc)
    }

    pub fn object(value: HashMap<JsonString, JsonValue>, loc: Location) -> JsonValue {
        Self::new(ValueKind::Object(JsonObject::new(value)), loc)
    }
}
