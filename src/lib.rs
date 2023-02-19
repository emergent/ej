mod parser;

use parser::Json;
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct ParseError {
    pos: usize,
    kind: ParseErrorKind,
}

#[derive(Debug)]
pub enum ParseErrorKind {
    Lex,
    Syntax,
    Number,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "position: {}, reason: {:?}", self.pos, self.kind)
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
    }
}
