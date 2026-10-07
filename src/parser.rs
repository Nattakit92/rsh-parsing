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

pub fn parsing(s: String) -> Result<ActionList,String>{
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
        let a = parsing(String::from("ls"));
        assert_eq!("Command(ls)",a.unwrap().display());
        let b = parsing(String::from("cargo install rsh-crate"));
        assert_eq!("Command(cargo) Arg[install] Arg[rsh-crate]",b.unwrap().display());

        let c = parsing(String::from("echo \"Hello World!\""));
        assert_eq!("Command(echo) Arg[Hello World!]",c.unwrap().display());
        let d = parsing(String::from("echo \'\"Hello World!\"\'"));
        assert_eq!("Command(echo) Arg[\"Hello World!\"]",d.unwrap().display());
        let e = parsing(String::from("echo \"Hello \\\"World\\\"\""));
        assert_eq!("Command(echo) Arg[Hello \"World\"]",e.unwrap().display());
        let f = parsing(String::from("echo \\$100"));
        assert_eq!("Command(echo) Arg[$100]",f.unwrap().display());
        let g = parsing(String::from("echo \\\\"));
        assert_eq!("Command(echo) Arg[\\]",g.unwrap().display());
    }

    #[test]
    fn err_handl_case1(){
        let a = parsing(String::from("\""));
        assert_eq!("DoubleQuote: open but never closed",a.err().unwrap());
        let b = parsing(String::from("echo\'helloworld"));
        assert_eq!("SingleQuote: open but never closed",b.err().unwrap());
    }

    //test for variables
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
    fn err_handl_case2(){
        let a = parsing(String::from("echo ${var 1}"));
        assert_eq!("invalid char: space",a.err().unwrap());
        let b = parsing(String::from("echo ${100%}"));
        assert_eq!("invalid name: cannot start with number",b.err().unwrap());
    }

    //test for basic expansion
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

    //test for command substitution
    #[test]
    fn case4(){
        let a = parsing(String::from("cat $(seq 1 1 10)"));
        assert_eq!("Command(cat) Arg[ComSub(Command(seq) Arg[1] Arg[1] Arg[10])]",a.unwrap().display());
        let b = parsing(String::from("cat file$(seq 1 1 10)"));
        assert_eq!("Command(cat) Arg[file,ComSub(Command(seq) Arg[1] Arg[1] Arg[10])]",b.unwrap().display());
        let c = parsing(String::from("cat file$(seq 1 1 10).txt"));
        assert_eq!("Command(cat) Arg[file,ComSub(Command(seq) Arg[1] Arg[1] Arg[10]),.txt]",c.unwrap().display());
    }

    //test for arithmatic expansion
    #[test]
    fn case5(){
        let a = parsing(String::from("echo $((1&2&3))"));
        //1&2&3
        assert_eq!("Command(echo) Arg[ArithEx(BitAnd[Int(1),Int(2),Int(3)])]",a.unwrap().display());
        let b = parsing(String::from("echo $((1 & 2&3))"));
        //1&2&3
        assert_eq!("Command(echo) Arg[ArithEx(BitAnd[Int(1),Int(2),Int(3)])]",b.unwrap().display());
        let c = parsing(String::from("echo $((1+2*3))"));
        //1+(2*3)
        assert_eq!("Command(echo) Arg[ArithEx(Add[Int(1),Mult[Int(2),Int(3)]])]",c.unwrap().display());
        let d = parsing(String::from("echo $((1*2/3))"));
        //(1*2)/3
        assert_eq!("Command(echo) Arg[ArithEx(Div[Mult[Int(1),Int(2)],Int(3)])]",d.unwrap().display());
        let e = parsing(String::from("echo $((1+1 == 2))"));
        //(1+1) == 2
        assert_eq!("Command(echo) Arg[ArithEx(Eq[Add[Int(1),Int(1)],Int(2)])]",e.unwrap().display());
        let f = parsing(String::from("echo $((1+1 > 2))"));
        //(1+1) == 2
        assert_eq!("Command(echo) Arg[ArithEx(Greater[Add[Int(1),Int(1)],Int(2)])]",f.unwrap().display());
        let g = parsing(String::from("echo $((1+1 == 2 && 9+10 == 21))"));
        //((1+1) == 2) && ((9+10) == 21)
        assert_eq!("Command(echo) Arg[ArithEx(And[Eq[Add[Int(1),Int(1)],Int(2)],Eq[Add[Int(9),Int(10)],Int(21)]])]",g.unwrap().display());
        let h = parsing(String::from("echo $((1+1 == 2 || 9+10 == 21))"));
        //((1+1) == 2) || ((9+10) == 21)
        assert_eq!("Command(echo) Arg[ArithEx(Or[Eq[Add[Int(1),Int(1)],Int(2)],Eq[Add[Int(9),Int(10)],Int(21)]])]",h.unwrap().display());
        let i = parsing(String::from("echo $((P*(1+r)**t))"));
        //P((1+r)**t)
        assert_eq!("Command(echo) Arg[ArithEx(Mult[Var(P),Pow[Add[Int(1),Var(r)],Var(t)]])]",i.unwrap().display());
    }

    //test for command chaining
    #[test]
    fn case6(){
        let a = parsing(String::from("echo hello && echo world"));
        assert_eq!("And(Command(echo) Arg[hello] ) Command(echo) Arg[world]",a.unwrap().display());
        let b = parsing(String::from("cargo install rsh &"));
        assert_eq!("Background(Command(cargo) Arg[install] Arg[rsh] )",b.unwrap().display());
        let c = parsing(String::from("echo hello || echo world"));
        assert_eq!("Or(Command(echo) Arg[hello] ) Command(echo) Arg[world]",c.unwrap().display());
        let d = parsing(String::from("cat file.txt | grep helloworld"));
        assert_eq!("Pipe(Command(cat) Arg[file.txt] ) Command(grep) Arg[helloworld]",d.unwrap().display());
        let e = parsing(String::from("echo hello ; echo world"));
        assert_eq!("Sepr(Command(echo) Arg[hello] ) Command(echo) Arg[world]",e.unwrap().display());
    }
}
