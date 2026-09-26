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
    Escape(Box<State>),
}

pub fn parsing(s: String) -> Result<ActionList,String>{
    let mut result = ActionList::new();
    let mut s_chars = s.chars();
    let mut str_slice = String::new();
    let mut state = State::Normal;
    while let Some(c) = s_chars.next() {
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
            }
            _ => str_slice.push(c),
        }
    }
    if !str_slice.is_empty(){
        push_str(str_slice, &mut result);
    }
    Ok(result)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn case1(){
        let a = parsing(String::from("cargo install rsh-crate"));
        assert_eq!("Command(cargo) Arg[install] Arg[rsh-crate]",a.unwrap().display());
        let b = parsing(String::from("echo \"Hello World!\""));
        assert_eq!("Command(echo) Arg[Hello World!]",b.unwrap().display());
        let c = parsing(String::from("echo \'\"Hello World!\"\'"));
        assert_eq!("Command(echo) Arg[\"Hello World!\"]",c.unwrap().display());
        let d = parsing(String::from("echo \"Hello \\\"World\\\"\""));
        assert_eq!("Command(echo) Arg[Hello \"World\"]",d.unwrap().display());
        let e = parsing(String::from("echo \\$100"));
        assert_eq!("Command(echo) Arg[$100]",e.unwrap().display());
        let e = parsing(String::from("echo \\\\"));
        assert_eq!("Command(echo) Arg[\\]",e.unwrap().display());
    }

    #[test]
    fn case2(){
        let a = parsing(String::from("echo $var"));
        assert_eq!("Command(echo) Arg[Var(var)]",a.unwrap().display());
        let b = parsing(String::from("echo ${var1}helloworld"));
        assert_eq!("Command(echo) Arg[Var(var1),helloworld]",b.unwrap().display());
        let c = parsing(String::from("echo ${var1}${var2} helloworld"));
        assert_eq!("Command(echo) Arg[Var(var1),Var(var2)] Arg[helloworld]",c.unwrap().display());
    }

    #[test]
    fn case3(){
        let a_str = String::from("cat $dir_var");
        let a = parsing(a_str.clone());
        assert_eq!("Command(cat) Arg[Var(dir_var)]",a.unwrap().display());
        let b = parsing(format!("ls $({})",a_str));
        assert_eq!("Command(ls) Arg[ComSub(Command(cat) Arg[Var(dir_var)])]",b.unwrap().display());
        let c = parsing(String::from("echo \"Hello! ${name}\""));
        assert_eq!("Command(echo) Arg[Hello! ,Var(name)]",c.unwrap().display());
        let c = parsing(String::from("echo \'Hello! ${name}\'"));
        assert_eq!("Command(echo) Arg[Hello! ${name}]",c.unwrap().display());
    }
}
