mod parser;

use parser::Json;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ParseError {
    Lex,
    Syntax,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParseError {}

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
