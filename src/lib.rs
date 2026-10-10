//! `rsh_parser` is a library created to help parse the input for [`RSH`](https://crates.io/crates/rsh-crate)
//! # Quick Start
//! ```
//! // create a new ActionList with command echo "Hello!${name}"
//! let parser = rsh_parser::parser::parse("echo \"Hello!${name}\"");
//! assert!(parser.is_ok());
//! let mut act_list = parser.unwrap();
//! // we can visualize the value in act_list using display()
//! assert_eq!("Command(echo) Arg[Hello!,Var(name)]", act_list.display());
//! // we can also extract value from act_list using next()
//! use rsh_parser::types::*;
//! assert_eq!(Action::Command("echo".to_string()), act_list.next().unwrap());
//! assert_eq!(Action::Arg(vec![ArgPart::Literal("Hello!".to_string()),ArgPart::Var("name".to_string())]), act_list.next().unwrap());
//! ```

#![allow(dead_code)]
pub mod parser;
pub mod types;
mod parsers;


fn valid_var_char(c: char) -> bool{
    c.is_alphanumeric() || c == '_' || c=='-'
}

/// Checks if the value is a valid variable name.
///
/// Naming Rules:
/// * Starting characters: Cannot start with hyphens(`-`) or number(`0-9`).
/// * Allowed characters: Can contain any letters, number, underscore(`_`), and hyphens(`_`).
/// * No special characters: Symbol like hashes(`#`), backslash(`\`) or at-signs(`@`) are invalid.
pub fn valid_var_name(s: &str) -> Result<(),String>{
    if s.is_empty(){
        return Err(String::from("invalid name: cannot be empty"));
    }
    let mut s_chars = s.chars();
    let first_char = s_chars.clone().next().unwrap();
    if first_char.is_ascii_digit(){
        return Err(String::from("invalid name: cannot start with number"));
    }
    if first_char == '-'{
        return Err(String::from("invalid name: cannot start with hyphens"));
    }
    while let Some(c) = s_chars.next() {
        if c == ' '{
            return Err(String::from("invalid char: space"));
        }
        if !valid_var_char(c){
            return Err(format!("invalid char: {}",c))
        }
    }
    Ok(())
}
