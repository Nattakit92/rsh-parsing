//! `rsh_parser` is a library created to help parse the input for [`RSH`](https://crates.io/crates/rsh-crate)
//! # Quick Start
//! ```
//! // create a new ActionList with command echo "Hello!${name}"
//! let parser = rsh_parser::parse("echo \"Hello!${name}\"");
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

use crate::types::{Action, ActionList, ArgPart};
use crate::parsers::expand;

fn push_str(s: String, result: &mut ActionList){
    if result.has_com(){
        result.push_arg(ArgPart::Literal(s));
    }else{
        result.set_tail(Action::Command(s));
    }
}

enum State{
    Normal,
    DoubleQuote,
    SingleQuote,
    And, Pipe,
    Escape(Box<State>),
}

pub fn parse(s: &str) -> Result<ActionList,String>{
    let mut result = ActionList::new();
    let mut s_chars = s.chars();
    let mut str_slice = String::new();
    let mut state = State::Normal;
    while let Some(c) = s_chars.next() {
        match c {
            '&' => match state {
                State::Normal => {
                    state = State::And;
                    continue;
                },
                State::And => {
                    state = State::Normal;
                    result = ActionList::and(result);
                    result.push_none();
                    continue;
                },
                _ => (),
            },
            '|' => match state {
                State::Normal => {
                    state = State::Pipe;
                    continue;
                },
                State::Pipe => {
                    state = State::Normal;
                    result = ActionList::or(result);
                    result.push_none();
                    continue;
                },
                _ => (),
            },
            _ => match state {
                State::And => {
                    state = State::Normal;
                    result = ActionList::background(result);
                    result.push_none();
                    continue;
                },
                State::Pipe => {
                    state = State::Normal;
                    result = ActionList::pipe(result);
                    result.push_none();
                    continue;
                },
                _ => (),
            }
        }
        match c {
            ' ' if matches!(state, State::Normal) => {
                if !str_slice.is_empty(){
                    push_str(str_slice.clone(), &mut result);
                    str_slice = String::new();
                }
                result.push_none();
            },
            '$' if matches!(state, State::Normal | State::DoubleQuote) => {
                if !str_slice.is_empty(){
                    push_str(str_slice.clone(), &mut result);
                    str_slice = String::new();
                }
                match expand::expand(&mut s_chars) {
                    Ok(x) => result.push_arg(x),
                    Err(e) => return Err(e)
                }
            },
            '$' if matches!(state, State::Escape(_)) => {
                str_slice.push(c);
                if let State::Escape(x) = state{
                    state = *x;
                }
            }
            '\"' => {
                match state {
                    State::Normal => state = State::DoubleQuote,
                    State::DoubleQuote => state = State::Normal,
                    State::SingleQuote => str_slice.push(c),
                    State::Escape(x) => {
                        str_slice.push(c);
                        state = *x
                    },
                    _ => ()
                }
            },
            '\'' => {
                match state {
                    State::Normal => state = State::SingleQuote,
                    State::DoubleQuote => str_slice.push(c),
                    State::SingleQuote => state = State::Normal,
                    State::Escape(x) => {
                        str_slice.push(c);
                        state = *x
                    },
                    _ => ()
                }
            },
            ';' => {
                match state {
                    State::Normal => result = ActionList::sepr(result),
                    _ => str_slice.push(c),
                }
            },
            '\\' => {
                match state {
                    State::Escape(x) => {
                        str_slice.push(c);
                        state = *x
                    },
                    _ => state = State::Escape(Box::new(state)),
                }
            },
            _ => str_slice.push(c),
        }
    }
    match state {
        State::And => result = ActionList::background(result),
        State::Pipe => result = ActionList::pipe(result),
        State::SingleQuote => return Err(String::from("SingleQuote: open but never closed")),
        State::DoubleQuote => return Err(String::from("DoubleQuote: open but never closed")),
        _ => (),
    }
    if !str_slice.is_empty(){
        push_str(str_slice, &mut result);
    }
    Ok(result)
}

#[cfg(test)]
mod parsing{
    use super::*;

    //test for basic parsing (eg. tokeniser, escape char)
    #[test]
    fn case1(){
        let a = parse("ls");
        assert_eq!("Command(ls)",a.unwrap().display());
        let b = parse("cargo install rsh-crate");
        assert_eq!("Command(cargo) Arg[install] Arg[rsh-crate]",b.unwrap().display());

        let c = parse("echo \"Hello World!\"");
        assert_eq!("Command(echo) Arg[Hello World!]",c.unwrap().display());
        let d = parse("echo \'\"Hello World!\"\'");
        assert_eq!("Command(echo) Arg[\"Hello World!\"]",d.unwrap().display());
        let e = parse("echo \"Hello \\\"World\\\"\"");
        assert_eq!("Command(echo) Arg[Hello \"World\"]",e.unwrap().display());
        let f = parse("echo \\$100");
        assert_eq!("Command(echo) Arg[$100]",f.unwrap().display());
        let g = parse("echo \\\\");
        assert_eq!("Command(echo) Arg[\\]",g.unwrap().display());
    }

    #[test]
    fn err_handl_case1(){
        let a = parse("\"");
        assert_eq!("DoubleQuote: open but never closed",a.err().unwrap());
        let b = parse("echo\'helloworld");
        assert_eq!("SingleQuote: open but never closed",b.err().unwrap());
    }

    //test for variables
    #[test]
    fn case2(){
        let a = parse("echo $var");
        assert_eq!("Command(echo) Arg[Var(var)]",a.unwrap().display());
        let b = parse("echo ${var1}helloworld");
        assert_eq!("Command(echo) Arg[Var(var1),helloworld]",b.unwrap().display());
        let c = parse("echo ${var1}${var2} helloworld");
        assert_eq!("Command(echo) Arg[Var(var1),Var(var2)] Arg[helloworld]",c.unwrap().display());
    }

    #[test]
    fn err_handl_case2(){
        let a = parse("echo ${var 1}");
        assert_eq!("invalid char: space",a.err().unwrap());
        let b = parse("echo ${100%}");
        assert_eq!("invalid name: cannot start with number",b.err().unwrap());
    }

    //test for basic expansion
    #[test]
    fn case3(){
        let a_str = "cat $dir_var";
        let a = parse(a_str);
        assert_eq!("Command(cat) Arg[Var(dir_var)]",a.unwrap().display());
        let b = parse(&format!("ls $({})",a_str));
        assert_eq!("Command(ls) Arg[ComSub(Command(cat) Arg[Var(dir_var)])]",b.unwrap().display());

        let c = parse("echo \"Hello! ${name}\"");
        assert_eq!("Command(echo) Arg[Hello! ,Var(name)]",c.unwrap().display());
        let c = parse("echo \'Hello! ${name}\'");
        assert_eq!("Command(echo) Arg[Hello! ${name}]",c.unwrap().display());
    }

    //test for command substitution
    #[test]
    fn case4(){
        let a = parse("cat $(seq 1 1 10)");
        assert_eq!("Command(cat) Arg[ComSub(Command(seq) Arg[1] Arg[1] Arg[10])]",a.unwrap().display());
        let b = parse("cat file$(seq 1 1 10)");
        assert_eq!("Command(cat) Arg[file,ComSub(Command(seq) Arg[1] Arg[1] Arg[10])]",b.unwrap().display());
        let c = parse("cat file$(seq 1 1 10.txt)");
        assert_eq!("Command(cat) Arg[file,ComSub(Command(seq) Arg[1] Arg[1] Arg[10.txt])]",c.unwrap().display());
    }

    //test for arithmatic expansion
    #[test]
    fn case5(){
        let a = parse("echo $((1&2&3))");
        //1&2&3
        assert_eq!("Command(echo) Arg[ArithEx(BitAnd[Int(1),Int(2),Int(3)])]",a.unwrap().display());
        let b = parse("echo $((1 & 2&3))");
        //1&2&3
        assert_eq!("Command(echo) Arg[ArithEx(BitAnd[Int(1),Int(2),Int(3)])]",b.unwrap().display());
        let c = parse("echo $((1+2*3))");
        //1+(2*3)
        assert_eq!("Command(echo) Arg[ArithEx(Add[Int(1),Mult[Int(2),Int(3)]])]",c.unwrap().display());
        let d = parse("echo $((1*2/3))");
        //(1*2)/3
        assert_eq!("Command(echo) Arg[ArithEx(Div[Mult[Int(1),Int(2)],Int(3)])]",d.unwrap().display());
        let e = parse("echo $((1+1 == 2))");
        //(1+1) == 2
        assert_eq!("Command(echo) Arg[ArithEx(Eq[Add[Int(1),Int(1)],Int(2)])]",e.unwrap().display());
        let f = parse("echo $((1+1 > 2))");
        //(1+1) == 2
        assert_eq!("Command(echo) Arg[ArithEx(Greater[Add[Int(1),Int(1)],Int(2)])]",f.unwrap().display());
        let g = parse("echo $((1+1 == 2 && 9+10 == 21))");
        //((1+1) == 2) && ((9+10) == 21)
        assert_eq!("Command(echo) Arg[ArithEx(And[Eq[Add[Int(1),Int(1)],Int(2)],Eq[Add[Int(9),Int(10)],Int(21)]])]",g.unwrap().display());
        let h = parse("echo $((1+1 == 2 || 9+10 == 21))");
        //((1+1) == 2) || ((9+10) == 21)
        assert_eq!("Command(echo) Arg[ArithEx(Or[Eq[Add[Int(1),Int(1)],Int(2)],Eq[Add[Int(9),Int(10)],Int(21)]])]",h.unwrap().display());
        let i = parse("echo $((P*(1+r)**t))");
        //P((1+r)**t)
        assert_eq!("Command(echo) Arg[ArithEx(Mult[Var(P),Pow[Add[Int(1),Var(r)],Var(t)]])]",i.unwrap().display());
    }

    //test for command chaining
    #[test]
    fn case6(){
        let a = parse("echo hello && echo world");
        assert_eq!("And(Command(echo) Arg[hello]) Command(echo) Arg[world]",a.unwrap().display());
        let b = parse("cargo install rsh &");
        assert_eq!("Background(Command(cargo) Arg[install] Arg[rsh])",b.unwrap().display());
        let c = parse("echo hello || echo world");
        assert_eq!("Or(Command(echo) Arg[hello]) Command(echo) Arg[world]",c.unwrap().display());
        let d = parse("cat file.txt | grep helloworld");
        assert_eq!("Pipe(Command(cat) Arg[file.txt]) Command(grep) Arg[helloworld]",d.unwrap().display());
        let e = parse("echo hello ; echo world");
        assert_eq!("Sepr(Command(echo) Arg[hello]) Command(echo) Arg[world]",e.unwrap().display());
    }
}
