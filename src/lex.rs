use crate::ParseError;
use std::collections::{HashSet, VecDeque};

const JSON_QUOTE: char = '"';
const JSON_NUMBER_CHARS: &str = "0123456789-e.";
const LEN_TRUE: usize = 4;
const LEN_FALSE: usize = 5;
const LEN_NULL: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Comma,        // ,
    Colon,        // :
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Null,
}

pub fn lex(s: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens = vec![];
    let mut chars = s.chars().collect::<VecDeque<char>>();

    while !chars.is_empty() {
        //println!("lex loop: {:?}", chars);

        let s_opt = lex_string(&mut chars)?;
        if let Some(s) = s_opt {
            tokens.push(s);
            continue;
        }

        let n_opt = lex_number(&mut chars)?;
        if let Some(n) = n_opt {
            tokens.push(n);
            continue;
        }

        let b_opt = lex_bool(&mut chars)?;
        if let Some(b) = b_opt {
            tokens.push(b);
            continue;
        }

        let null_opt = lex_null(&mut chars)?;
        if let Some(null) = null_opt {
            tokens.push(null);
            continue;
        }

        if let Some(c) = chars.pop_front() {
            let t = match c {
                ' ' | '\t' | '\n' | '\r' => continue,
                ',' => Token::Comma,
                ':' => Token::Colon,
                '[' => Token::LeftBracket,
                ']' => Token::RightBracket,
                '{' => Token::LeftBrace,
                '}' => Token::RightBrace,
                _ => return Err(ParseError::Lex),
            };
            tokens.push(t);
        } else {
            break;
        }
    }

    Ok(tokens)
}

fn lex_string(chars: &mut VecDeque<char>) -> Result<Option<Token>, ParseError> {
    match chars.front() {
        Some(c) if *c == JSON_QUOTE => {
            chars.pop_front();
        }
        _ => return Ok(None),
    }

    let mut s = String::new();
    while let Some(c) = chars.pop_front() {
        if c == JSON_QUOTE {
            return Ok(Some(Token::String(s)));
        } else {
            s.push(c);
        }
    }

    Err(ParseError::Lex)
}

fn lex_number(chars: &mut VecDeque<char>) -> Result<Option<Token>, ParseError> {
    let num_chars = JSON_NUMBER_CHARS.chars().collect::<HashSet<char>>();

    let mut s = String::new();
    while let Some(c) = chars.pop_front() {
        if num_chars.contains(&c) {
            s.push(c);
        } else {
            chars.push_front(c);
            break;
        }
    }

    if s.is_empty() {
        return Ok(None);
    }

    if let Ok(i) = s.parse::<i64>() {
        return Ok(Some(Token::Integer(i)));
    } else if let Ok(f) = s.parse::<f64>() {
        return Ok(Some(Token::Float(f)));
    }

    Err(ParseError::Lex)
}

fn lex_bool(chars: &mut VecDeque<char>) -> Result<Option<Token>, ParseError> {
    if chars.len() > LEN_TRUE
        && chars[0] == 't'
        && chars[1] == 'r'
        && chars[2] == 'u'
        && chars[3] == 'e'
    {
        for _ in 0..LEN_TRUE {
            chars.pop_front();
        }
        return Ok(Some(Token::Bool(true)));
    } else if chars.len() > LEN_FALSE
        && chars[0] == 'f'
        && chars[1] == 'a'
        && chars[2] == 'l'
        && chars[3] == 's'
        && chars[4] == 'e'
    {
        for _ in 0..LEN_FALSE {
            chars.pop_front();
        }
        return Ok(Some(Token::Bool(false)));
    }

    Ok(None)
}

fn lex_null(chars: &mut VecDeque<char>) -> Result<Option<Token>, ParseError> {
    if chars.len() > LEN_NULL
        && chars[0] == 'n'
        && chars[1] == 'u'
        && chars[2] == 'l'
        && chars[3] == 'l'
    {
        for _ in 0..LEN_NULL {
            chars.pop_front();
        }
        return Ok(Some(Token::Null));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::Token as T;
    use super::*;

    fn lx(s: &str, expected: &[Token]) {
        let res = lex(s);
        println!("{:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, expected.to_vec());
        println!("JSON str: {}", s);
        println!("tokens: {:#?}", res);
    }

    #[test]
    fn test_lex() {
        lx(r#""#, &[]);
        lx(r#"{"#, &[T::LeftBrace]);
        lx(r#"{}"#, &[T::LeftBrace, T::RightBrace]);
        lx(r#"[]"#, &[T::LeftBracket, T::RightBracket]);
        lx(
            r#"{"a":"b"}"#,
            &[
                T::LeftBrace,
                T::String("a".into()),
                T::Colon,
                T::String("b".into()),
                T::RightBrace,
            ],
        );
        lx(
            r#"{"a":100}"#,
            &[
                T::LeftBrace,
                T::String("a".into()),
                T::Colon,
                T::Integer(100),
                T::RightBrace,
            ],
        );
        lx(
            r#"{"a":100.123}"#,
            &[
                T::LeftBrace,
                T::String("a".into()),
                T::Colon,
                T::Float(100.123),
                T::RightBrace,
            ],
        );
        lx(
            r#"{"a":true}"#,
            &[
                T::LeftBrace,
                T::String("a".into()),
                T::Colon,
                T::Bool(true),
                T::RightBrace,
            ],
        );
        lx(
            r#"{"a":false}"#,
            &[
                T::LeftBrace,
                T::String("a".into()),
                T::Colon,
                T::Bool(false),
                T::RightBrace,
            ],
        );
        lx(
            r#"{"a":null}"#,
            &[
                T::LeftBrace,
                T::String("a".into()),
                T::Colon,
                T::Null,
                T::RightBrace,
            ],
        );
        lx(
            r#"{true : null, false : true}"#,
            &[
                T::LeftBrace,
                T::Bool(true),
                T::Colon,
                T::Null,
                T::Comma,
                T::Bool(false),
                T::Colon,
                T::Bool(true),
                T::RightBrace,
            ],
        );
        lx(
            r#"]true"abc":null false{]"#,
            &[
                T::RightBracket,
                T::Bool(true),
                T::String("abc".into()),
                T::Colon,
                T::Null,
                T::Bool(false),
                T::LeftBrace,
                T::RightBracket,
            ],
        )
    }
}
