use crate::{Json, ParseError};
use std::collections::{HashMap, HashSet};

const JSON_QUOTE: u8 = b'"';
const JSON_NUMBER_CHARS: &str = "0123456789-e.";
const JSON_WHITESPACE_CHARS: &str = " \n\r\t";
const LEN_TRUE: usize = 4;
const LEN_FALSE: usize = 5;
const LEN_NULL: usize = 4;

#[derive(Debug)]
pub struct Location(usize, usize);

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

pub type Value = Annotation<ValueKind>;

impl Value {
    pub fn kind(&self) -> &ValueKind {
        &self.value
    }

    fn null(loc: Location) -> Value {
        Self::new(ValueKind::Null, loc)
    }

    fn bool(value: bool, loc: Location) -> Value {
        Self::new(ValueKind::Bool(value), loc)
    }

    fn number_int(value: i64, loc: Location) -> Value {
        Self::new(ValueKind::Number(Number::Integer(value)), loc)
    }

    fn number_float(value: f64, loc: Location) -> Value {
        Self::new(ValueKind::Number(Number::Float(value)), loc)
    }

    fn string(value: String, loc: Location) -> Value {
        Self::new(ValueKind::String(value), loc)
    }

    fn array(value: Vec<Value>, loc: Location) -> Value {
        Self::new(ValueKind::Array(value), loc)
    }

    fn object(value: HashMap<String, Value>, loc: Location) -> Value {
        Self::new(ValueKind::Object(value), loc)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

impl ToString for Number {
    fn to_string(&self) -> String {
        match self {
            Number::Integer(i) => i.to_string(),
            Number::Float(f) => f.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum ValueKind {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    String(String),
    Number(Number),
    Bool(bool),
    Null,
}

pub fn parse_str(s: &str) -> Result<Json, ParseError> {
    let b = s.as_bytes();
    let parser = Parser::new(b);
    parser.parse()
}

#[derive(Debug)]
pub struct Parser<'a> {
    pos: usize,
    bytes: &'a [u8],
    values: Vec<Value>,
    number_chars: HashSet<u8>,
    whitespace: HashSet<u8>,
}

impl<'a> Parser<'a> {
    pub fn new(b: &'a [u8]) -> Self {
        Self {
            pos: 0,
            bytes: b,
            values: vec![],
            number_chars: JSON_NUMBER_CHARS.chars().map(|x| x as u8).collect(),
            whitespace: JSON_WHITESPACE_CHARS.chars().map(|x| x as u8).collect(),
        }
    }

    pub fn parse(mut self) -> Result<Json, ParseError> {
        while self.pos < self.bytes.len() {
            if self.skip_whitespace().is_err() {
                break;
            }

            let val = self.parse_bytes()?;
            self.values.push(val);
        }

        Ok(Json(self.values))
    }

    fn parse_bytes(&mut self) -> Result<Value, ParseError> {
        self.skip_whitespace()?;

        let value = match self.bytes[self.pos] {
            b'{' => self.parse_object()?,
            b'[' => self.parse_array()?,
            b'"' => self.parse_string()?,
            b'-' | b'0'..=b'9' => self.parse_number()?,
            b't' | b'f' => self.parse_bool()?,
            b'n' => self.parse_null()?,
            _ => return Err(ParseError::syntax(self.pos)),
        };

        Ok(value)
    }

    fn parse_null(&mut self) -> Result<Value, ParseError> {
        let i = self.pos;
        if self.bytes[i..i + LEN_NULL] == [b'n', b'u', b'l', b'l'] {
            self.pos += LEN_NULL;
            return Ok(Value::null(Location(i, i + LEN_NULL)));
        }
        Err(ParseError::syntax(self.pos))
    }

    fn parse_bool(&mut self) -> Result<Value, ParseError> {
        let i = self.pos;
        if self.bytes[i..i + LEN_TRUE] == [b't', b'r', b'u', b'e'] {
            self.pos += LEN_TRUE;
            return Ok(Value::bool(true, Location(i, i + LEN_TRUE)));
        } else if self.bytes[i..i + LEN_FALSE] == [b'f', b'a', b'l', b's', b'e'] {
            self.pos += LEN_FALSE;
            return Ok(Value::bool(false, Location(i, i + LEN_FALSE)));
        }
        Err(ParseError::syntax(self.pos))
    }

    fn parse_number(&mut self) -> Result<Value, ParseError> {
        let mut cursor = 0;
        while self.pos + cursor < self.bytes.len()
            && self.match_number_token(&self.bytes[self.pos + cursor])
        {
            cursor += 1;
        }

        let num_slice = self.bytes[self.pos..self.pos + cursor]
            .iter()
            .map(|x| *x as char)
            .collect::<String>();

        let loc = Location(self.pos, self.pos + cursor);
        if let Ok(i) = num_slice.parse::<i64>() {
            self.pos += cursor;
            return Ok(Value::number_int(i, loc));
        } else if let Ok(f) = num_slice.parse::<f64>() {
            self.pos += cursor;
            return Ok(Value::number_float(f, loc));
        }
        Err(ParseError::number(self.pos))
    }

    fn parse_string(&mut self) -> Result<Value, ParseError> {
        let start_pos = self.pos;
        let mut closed = false;
        self.pos += 1; // skip first '"'

        let mut cursor = 0;
        while self.pos + cursor < self.bytes.len() {
            if self.bytes[self.pos + cursor] == JSON_QUOTE {
                // FIXME: when escaped
                closed = true;
                break;
            }
            cursor += 1;
        }

        if !closed {
            return Err(ParseError::syntax(self.pos));
        }

        let s = self.bytes[self.pos..self.pos + cursor]
            .iter()
            .map(|x| *x as char)
            .collect::<String>();
        self.pos += cursor + 1; //skip closing '"'

        Ok(Value::string(s, Location(start_pos, self.pos)))
    }

    fn parse_array(&mut self) -> Result<Value, ParseError> {
        let start_pos = self.pos;
        self.pos += 1; // skip first '['

        let mut array = vec![];

        self.skip_whitespace()?;

        if self.bytes[self.pos] == b']' {
            self.pos += 1; // skip closing ']'
            return Ok(Value::array(array, Location(start_pos, self.pos)));
        }

        loop {
            let val = self.parse_bytes()?;
            array.push(val);

            self.skip_whitespace()?;

            match self.bytes[self.pos] {
                b']' => {
                    self.pos += 1;
                    break;
                }
                b',' => {
                    self.pos += 1;
                    self.skip_whitespace()?;
                }
                _ => return Err(ParseError::syntax(self.pos)),
            }
        }

        Ok(Value::array(array, Location(start_pos, self.pos)))
    }

    fn parse_object(&mut self) -> Result<Value, ParseError> {
        let start_pos = self.pos;
        self.pos += 1; // skip first '{'

        let mut hm = HashMap::new();

        self.skip_whitespace()?;

        if self.bytes[self.pos] == b'}' {
            self.pos += 1; // skip closing ']'
            return Ok(Value::object(hm, Location(start_pos, self.pos)));
        }

        loop {
            let Value { value: ValueKind::String(key),..} = self.parse_string()? else {
                return Err(ParseError::syntax(self.pos));
            };

            self.skip_whitespace()?;

            if self.bytes[self.pos] != b':' {
                return Err(ParseError::syntax(self.pos));
            }
            self.pos += 1;

            self.skip_whitespace()?;

            let val = self.parse_bytes()?;

            hm.insert(key, val);

            self.skip_whitespace()?;

            match self.bytes[self.pos] {
                b'}' => {
                    self.pos += 1;
                    break;
                }
                b',' => {
                    self.pos += 1;
                    self.skip_whitespace()?;
                }
                _ => return Err(ParseError::syntax(self.pos)),
            }
        }

        Ok(Value::object(hm, Location(start_pos, self.pos)))
    }

    fn match_number_token(&self, c: &u8) -> bool {
        self.number_chars.contains(c)
    }

    fn skip_whitespace(&mut self) -> Result<(), ParseError> {
        while self.pos < self.bytes.len() {
            if self.whitespace.contains(&self.bytes[self.pos]) {
                self.pos += 1;
            } else {
                return Ok(());
            }
        }

        Err(ParseError::syntax(self.pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) {
        let res = parse_str(s);
        assert!(res.is_ok());
        let _res = res.unwrap();
        //println!("{:#?}", _res);
    }

    #[test]
    fn test_ok() {
        p(r#""#);

        p(r#"true"#);
        p(r#"false"#);

        p(r#"null"#);

        p(r#"100"#);
        p(r#"-100"#);
        p(r#"100e3"#);
        p(r#"-100e3"#);
        p(r#"100.1"#);
        p(r#"-1.2e3"#);

        p(r#""""#);
        p(r#""aaa""#);

        p(r#"[]"#);
        p(r#"[ ]"#);
        p(r#"[true, false, null, 1, "hoge"]"#);

        p(r#"{}"#);
        p(r#"{"a":100}"#);
        p(r#"{"a":"hoge"}"#);
        p(r#"{"a":true}"#);
        p(r#"{"a":false}"#);
        p(r#"{"a":null}"#);
        p(r#"  {"a":100, "b" : "hoge"   ,  "c"   :  true  } "#);
        p(r#"  {"a":100, "b" : "hoge" ,
          "c" : { "hoge": "aha" } } "#);

        p(r#"[{}]"#);
        p(r#"[  {"a":100, "b" : "hoge" ,
          "c" : { "hoge": "aha" } } ] "#);

        p(r#"[{}] {} {} {}"#);
        p(r#"[1,2,3] {"a":"b"} {"a":{"b":{"c":"d"}}} {}"#);
    }

    fn e(s: &str) {
        let res = parse_str(s);
        assert!(res.is_err());
    }

    #[test]
    fn test_ng() {
        e(r#","#);
        e(r#":"#);

        // array
        e(r#"["#);
        e(r#"[,]"#);
        e(r#"[:]]"#);
        e(r#"[1,"2",]"#);
        e(r#"[1,"2","#);
        e(r#"[1,"2",3 "#);

        // object loop
        e(r#"{"a": 1"#);
        e(r#"{"a": 1 ,"#);
        e(r#"{"a": 1 ,}"#);
        e(r#"{"a": 1, "b": , }"#);
        e(r#"{"a": 1, "b": 2"#);
    }
}
