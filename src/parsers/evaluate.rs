use std::{str::Chars};
use crate::types::ArgPart;

pub fn eval(s_chars: &mut Chars) -> Result<ArgPart, String>{
    while let Some(c) = s_chars.next() {
        todo!();
    }
    Err(String::from("unclosed parentheses: (( opened but never closed"))
}
