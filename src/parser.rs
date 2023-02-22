use crate::ParseError;
use std::collections::{HashMap, HashSet};

const JSON_QUOTE: u8 = b'"';
const JSON_NUMBER_CHARS: &str = "0123456789-e.";
const JSON_WHITESPACE_CHARS: &str = " \n\r\t";
const LEN_TRUE: usize = 4;
const LEN_FALSE: usize = 5;
const LEN_NULL: usize = 4;

#[derive(Debug)]
pub struct Json(Vec<Value>);

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
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

#[derive(Debug, Clone)]
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

        let value = match self.bytes[self.pos] as char {
            '{' => self.parse_object()?,
            '[' => self.parse_array()?,
            '"' => self.parse_string()?,
            '-' | '0'..='9' => self.parse_number()?,
            't' | 'f' => self.parse_bool()?,
            'n' => self.parse_null()?,
            _ => return Err(ParseError::syntax(self.pos)),
        };

        Ok(value)
    }

    fn parse_null(&mut self) -> Result<Value, ParseError> {
        let i = self.pos;
        if self.bytes[i..i + LEN_NULL] == [b'n', b'u', b'l', b'l'] {
            self.pos += LEN_NULL;
            return Ok(Value::Null);
        }
        Err(ParseError::syntax(self.pos))
    }

    fn parse_bool(&mut self) -> Result<Value, ParseError> {
        let i = self.pos;
        if self.bytes[i..i + LEN_TRUE] == [b't', b'r', b'u', b'e'] {
            self.pos += LEN_TRUE;
            return Ok(Value::Bool(true));
        } else if self.bytes[i..i + LEN_FALSE] == [b'f', b'a', b'l', b's', b'e'] {
            self.pos += LEN_FALSE;
            return Ok(Value::Bool(false));
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

        if let Ok(i) = num_slice.parse::<i64>() {
            self.pos += cursor;
            return Ok(Value::Number(Number::Integer(i)));
        } else if let Ok(f) = num_slice.parse::<f64>() {
            self.pos += cursor;
            return Ok(Value::Number(Number::Float(f)));
        }
        Err(ParseError::number(self.pos))
    }

    fn parse_string(&mut self) -> Result<Value, ParseError> {
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

        Ok(Value::String(s))
    }

    fn parse_array(&mut self) -> Result<Value, ParseError> {
        self.pos += 1; // skip first '['

        let mut array = vec![];

        self.skip_whitespace()?;

        if self.bytes[self.pos] == b']' {
            self.pos += 1; // skip closing ']'
            return Ok(Value::Array(array));
        }

        while self.pos < self.bytes.len() {
            let val = self.parse_bytes()?;
            array.push(val);

            self.skip_whitespace()?;

            match self.bytes[self.pos] {
                b']' => {
                    self.pos += 1;
                    return Ok(Value::Array(array));
                }
                b',' => {
                    self.pos += 1;
                    self.skip_whitespace()?;
                }
                _ => return Err(ParseError::syntax(self.pos)),
            }
        }

        Ok(Value::Null)
    }

    fn parse_object(&mut self) -> Result<Value, ParseError> {
        self.pos += 1; // skip first '{'

        let mut hm = HashMap::new();

        self.skip_whitespace()?;

        if self.bytes[self.pos] == b'}' {
            self.pos += 1; // skip closing ']'
            return Ok(Value::Object(hm));
        }

        while self.pos < self.bytes.len() {
            let Value::String(key) = self.parse_string()? else {
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

        Ok(Value::Object(hm))
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
        let res = res.unwrap();
        println!("{:#?}", res);
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
        e(r#"["#);
        e(r#"[,]"#);
        e(r#"[:]]"#);
    }
}
