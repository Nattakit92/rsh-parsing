use std::{str::Chars};

use crate::{valid_var_name};
use crate::{parsers::evaluate::arithexp};
use crate::{parse, types::ArgPart};

pub fn expand(s_chars: &mut Chars) -> Result<ArgPart,String>{
    let mut str_slice = String::new();
    match s_chars.clone().next() {
        Some('(') => return com_sub(s_chars),
        Some('{') => return variable(s_chars),
        Some(_) => (),
        None => return Ok(ArgPart::Literal(String::from('$')))
    }
    for c in s_chars {
        match c {
            ' ' => {
                break;
            }
            _ => str_slice.push(c),
        }
    }
    if let Err(e) = valid_var_name(&str_slice){
        return Err(e);
    }
    Ok(ArgPart::Var(str_slice))
}

fn variable(s_chars: &mut Chars) -> Result<ArgPart, String>{
    s_chars.next();
    let mut str_slice = String::new();
    for c in s_chars {
        match c {
            '}' => {
                if let Err(e) = valid_var_name(&str_slice){
                    return Err(e);
                }
                return Ok(ArgPart::Var(str_slice));
            },
            ' ' => {
                return Err(String::from("invalid char: space"));
            },
            _ => str_slice.push(c),
        }
    }
    Err(String::from("unclosed parentheses: { opened but never closed"))
}

fn com_sub(s_chars: &mut Chars) -> Result<ArgPart, String>{
    s_chars.next();
    let mut str_slice = String::new();
    match s_chars.next() {
        Some('(') => return arithexp(s_chars),
        Some(x) => str_slice.push(x),
        None => return Err(String::from("unclosed parentheses: ( opened but never closed"))
    }
    for c in s_chars {
        match c {
            ')' => {
                match parse(&str_slice) {
                    Ok(x) => return Ok(ArgPart::ComSub(x)),
                    Err(e) => return Err(e)
                }
            },
            _ => str_slice.push(c),
        }
    }
    Err(String::from("unclosed parentheses: ( opened but never closed"))
}
