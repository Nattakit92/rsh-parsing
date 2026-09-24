use std::{result, str::Chars};

use crate::types::{ArgPart,Calculate};

const BANNED: &[char] = &['{','}','(',')','[',']','@','$','%','.',',','/','\\'];

pub fn expand(s_chars: &mut Chars) -> Result<ArgPart,String>{
    let mut str_slice = String::new();
    let mut result: ArgPart;
    match s_chars.next() {
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
    result = ArgPart::Var(str_slice);
    Ok(result)
}

fn variable(s_chars: &mut Chars) -> Result<ArgPart, String>{
    let mut str_slice = String::new();
    while let Some(c) = s_chars.next() {
        match c {
            '}' => {

            },
            _ => str_slice.push(c),
        }
    }
    Err(String::from("unclosed : { opened but never closed"))
}
