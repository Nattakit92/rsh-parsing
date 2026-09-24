use std::{str::Chars};

use crate::types::{ArgPart};

const BANNED: &[char] = &['{','}','(',')','[',']','@','$','%','.',',','/','\\'];

pub fn expand(s_chars: &mut Chars) -> Result<ArgPart,String>{
    let mut str_slice = String::new();
    match s_chars.next() {
        Some('{') => return variable(s_chars),
        Some(x) => str_slice.push(x),
        None => return Ok(ArgPart::Literal(String::from('$')))
    }
    while let Some(c) = s_chars.next() {
        match c {
            ' ' => {
                break;
            }
            x if BANNED.contains(&x) => {
                return Err(format!("invalid char: {}",x));
            }
            _ => str_slice.push(c),
        }
    }
    Ok(ArgPart::Var(str_slice))
}

fn variable(s_chars: &mut Chars) -> Result<ArgPart, String>{
    let mut str_slice = String::new();
    while let Some(c) = s_chars.next() {
        match c {
            '}' => {
                return Ok(ArgPart::Var(str_slice));
            },
            _ => str_slice.push(c),
        }
    }
    Err(String::from("unclosed parentheses: { opened but never closed"))
}
