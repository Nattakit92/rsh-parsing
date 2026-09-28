#![allow(dead_code)]
pub mod parser;
pub mod types;
mod parsers;

pub const BANNED: &[char] = &['{','}','(',')','[',']','@','$','%','.',',','/','\\','!','\"','\''];
