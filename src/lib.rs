#![allow(dead_code)]
pub mod parser;
pub mod types;
mod parsers;

pub fn valid_var_char(c: char) -> bool{
    c.is_alphanumeric() || c == '_' || c=='-'
}
