mod lex;

use lex::{lex, Token};
use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    fmt,
};

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

#[derive(Debug)]
pub struct Json {
    inner: Option<Value>,
}

impl Json {
    pub fn into_inner(self) -> Option<Value> {
        self.inner
    }
}

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
    Null,
}

pub fn from_json_str(json_str: &str) -> Result<Json, ParseError> {
    let tokens = lex(json_str).map_err(|_| ParseError::Lex)?;
    println!("{:?}", tokens);

    let mut tokens = VecDeque::from(tokens);
    let inner = parse(&mut tokens)?;

    Ok(Json { inner })
}

fn parse(tokens: &mut VecDeque<Token>) -> Result<Option<Value>, ParseError> {
    if tokens.is_empty() {
        return Ok(None);
    }

    let root = match tokens.pop_front() {
        Some(Token::LeftBrace) => parse_object(tokens)?,
        Some(Token::LeftBracket) => parse_array(tokens)?,
        Some(Token::String(s)) => Value::String(s),
        Some(Token::Integer(i)) => Value::Number(Number::Integer(i)),
        Some(Token::Float(f)) => Value::Number(Number::Float(f)),
        Some(Token::Null) => Value::Null,
        _ => return Err(ParseError::Syntax),
    };

    if !tokens.is_empty() {
        eprintln!("unused token remains: {:?}", tokens);
        return Err(ParseError::Syntax);
    }

    Ok(Some(root))
}

fn parse_array(tokens: &mut VecDeque<Token>) -> Result<Value, ParseError> {
    let mut array = vec![];

    while let Some(token) = tokens.pop_front() {
        println!("token: {:?}, rest: {:?}", token, tokens);
        match token {
            Token::RightBracket => return Ok(Value::Array(array)),
            Token::String(s) => array.push(Value::String(s)),
            Token::Integer(i) => array.push(Value::Number(Number::Integer(i))),
            Token::Float(f) => array.push(Value::Number(Number::Float(f))),
            Token::Null => array.push(Value::Null),
            Token::LeftBrace => array.push(parse_object(tokens)?),
            Token::LeftBracket => array.push(parse_array(tokens)?),
            _ => return Err(ParseError::Syntax),
        }

        if let Some(sep) = tokens.pop_front() {
            match sep {
                Token::Comma => continue,
                Token::RightBracket => return Ok(Value::Array(array)),
                _ => return Err(ParseError::Syntax),
            }
        }
    }

    Err(ParseError::Syntax)
}

fn parse_object(tokens: &mut VecDeque<Token>) -> Result<Value, ParseError> {
    let mut hm = HashMap::new();

    while let Some(token) = tokens.pop_front() {
        println!("token: {:?}, rest: {:?}", token, tokens);
        let key = match token {
            Token::String(s) => s,
            Token::RightBrace => return Ok(Value::Object(hm)),
            _ => return Err(ParseError::Syntax),
        };

        let colon = tokens.pop_front();
        if !matches!(colon, Some(Token::Colon)) {
            return Err(ParseError::Syntax);
        }

        let val = match tokens.pop_front() {
            Some(Token::String(s)) => Value::String(s),
            Some(Token::Integer(i)) => Value::Number(Number::Integer(i)),
            Some(Token::Float(f)) => Value::Number(Number::Float(f)),
            Some(Token::Null) => Value::Null,
            Some(Token::LeftBrace) => parse_object(tokens)?,
            Some(Token::LeftBracket) => parse_array(tokens)?,
            _ => return Err(ParseError::Syntax),
        };

        hm.insert(key, val);

        if let Some(sep) = tokens.pop_front() {
            match sep {
                Token::Comma => continue,
                Token::RightBrace => return Ok(Value::Object(hm)),
                _ => return Err(ParseError::Syntax),
            }
        }
    }

    Err(ParseError::Syntax)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let res = from_json_str("{}");
        assert!(res.is_ok());
        println!("{:#?}", res.unwrap());

        let res = from_json_str("{");
        assert!(res.is_err());

        let json = r#"{
            "a" : 100,
            "bbbb": "hoge",
            "cc": { "c_inner": 200 },
            "dd": [100, 200, "aha"]
        }"#;
        let res = from_json_str(json);
        assert!(res.is_ok());
        println!("{:#?}", res.unwrap());
    }
}
