use std::{str::Chars};

use crate::{parsers::evaluate::arithexp};
use crate::{parser::parsing, types::ArgPart};

pub fn expand(s_chars: &mut Chars) -> Result<ArgPart,String>{
    let mut str_slice = String::new();
    match s_chars.next() {
        Some('(') => return com_sub(s_chars),
        Some('{') => return variable(s_chars),
        Some(x) if x.is_ascii_digit() => {
            return Err(String::from("invalid name: cannot start with number"))
        }
        Some(x) if x.is_alphabetic() => str_slice.push(x),
        Some(x) => return Err(format!("invalid char: {}",x)),
        None => return Ok(ArgPart::Literal(String::from('$')))
    }
    while let Some(c) = s_chars.next() {
        match c {
            ' ' => {
                break;
            }
            x if x.is_ascii_digit() => {
                return Err(String::from("invalid name: cannot start with number"));
            },
            x if x.is_alphabetic() => str_slice.push(c),
            _ => return Err(format!("invalid char: {}",c)),
        }
    }
    Ok(ArgPart::Var(str_slice))
}

fn variable(s_chars: &mut Chars) -> Result<ArgPart, String>{
    let mut str_slice = String::new();
    if let Some(c) = s_chars.next(){
        match c {
            '}' => {
                return Ok(ArgPart::Var(str_slice));
            },
            ' ' => {
                return Err(String::from("invalid char: space"));
            },
            x if x.is_ascii_digit() => {
                return Err(String::from("invalid name: cannot start with number"));
            },
            x if x.is_alphabetic() => str_slice.push(c),
            _ => return Err(format!("invalid char: {}",c))
        }
    }
    while let Some(c) = s_chars.next() {
        match c {
            '}' => {
                return Ok(ArgPart::Var(str_slice));
            },
            ' ' => {
                return Err(String::from("invalid char: space"));
            },
            x if x.is_alphanumeric() => str_slice.push(c),
            _ => return Err(format!("invalid char: {}",c))
        }
    }
    Err(String::from("unclosed parentheses: { opened but never closed"))
}

fn com_sub(s_chars: &mut Chars) -> Result<ArgPart, String>{
    let mut str_slice = String::new();
    match s_chars.next() {
        Some('(') => return arithexp(s_chars),
        Some(x) => str_slice.push(x),
        None => return Err(String::from("unclosed parentheses: ( opened but never closed"))
    }
    while let Some(c) = s_chars.next() {
        match c {
            ')' => {
                match parsing(str_slice) {
                    Ok(x) => return Ok(ArgPart::ComSub(x)),
                    Err(e) => return Err(e)
                }
            },
            _ => str_slice.push(c),
        }
    }
    Err(String::from("unclosed parentheses: ( opened but never closed"))
}
