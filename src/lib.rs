mod parser;

use parser::ValueKind;

use self::parser::Value;
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct Json(Vec<Value>);

impl Json {
    pub fn dump(&self) {
        for v in &self.0 {
            Self::dump_inner(None, v, 0);
        }
    }

    fn dump_inner(key: Option<&str>, value: &Value, level: usize) {
        for _ in 0..level {
            print!("  ");
        }

        let l = value.loc();
        let v = Self::format_value(value);

        match key {
            Some(k) => println!("{}: {} ({}, {})", k, v, l.start(), l.end()),
            None => println!("{} ({}, {})", v, l.start(), l.end()),
        }

        match value.kind() {
            ValueKind::Array(a) => {
                for ai in a {
                    Self::dump_inner(None, ai, level + 1);
                }
            }
            ValueKind::Object(hm) => {
                for (k, v) in hm {
                    Self::dump_inner(Some(k.as_str()), v, level + 1);
                }
            }
            _ => {}
        }
    }

    fn format_value(value: &Value) -> String {
        match value.kind() {
            ValueKind::Bool(b) => b.to_string(),
            ValueKind::Null => "null".into(),
            ValueKind::Object(_) => "Object".into(),
            ValueKind::Array(_) => "Array".into(),
            ValueKind::String(s) => s.to_owned(),
            ValueKind::Number(n) => n.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    pos: usize,
    kind: ParseErrorKind,
}

#[derive(Debug)]
pub enum ParseErrorKind {
    Syntax,
    Number,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse Error occurred at position {}, reason: {:?}",
            self.pos, self.kind
        )
    }
}

impl Error for ParseError {}

impl ParseError {
    pub fn syntax(pos: usize) -> Self {
        Self {
            pos,
            kind: ParseErrorKind::Syntax,
        }
    }

    pub fn number(pos: usize) -> Self {
        Self {
            pos,
            kind: ParseErrorKind::Number,
        }
    }
}

pub fn from_json_str(json_str: &str) -> Result<Json, ParseError> {
    let res = parser::parse_str(json_str)?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let res = from_json_str("{}");
        assert!(res.is_ok());

        let res = from_json_str("{");
        assert!(res.is_err());

        let json = r#"{
            "a" : 100,
            "bbbb": "hoge",
            "cc": { "c_inner": 200 },
            "dd": [100, 200, "aha"],
            "eeee" : true
        }"#;
        let res = from_json_str(json);
        assert!(res.is_ok());

        res.unwrap().dump();
    }
}
