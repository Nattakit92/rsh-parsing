use std::str::Chars;
use crate::types::{ArgPart, OpType, Operation, ValueTypes};
use crate::types::OpType::*;

const OPERATORS: [char;12] = ['&','|','^','=','!','>','<','+','-','*','/','%'];

enum State{
    Close, Not, Normal, Eq
}

pub fn arithexp(s_chars: &mut Chars) -> Result<ArgPart, String>{
    let mut optype = Null;
    let mut state = State::Normal;
    let mut value = String::new();
    let mut result = Operation::None;
    while let Some(c) = s_chars.next() {
        if c == '('{
            let bracket = arithpart(s_chars);
            match bracket {
                Ok(x) => merge_op(&mut optype, x, &mut result, &mut state),
                Err(e) => return Err(e),
            }
            continue;
        }
        if c == ')'{
            match state {
                State::Close => {
                    match push_op(&mut optype, &mut value, &mut result, &mut state) {
                        Ok(_) => return Ok(ArgPart::ArithEx(result)),
                        Err(e) => return Err(e)
                    }
                },
                State::Normal => state = State::Close,
                State::Not => return Err(String::from("arithmetic syntax error: invalid arithmetic operator (error token is \"!\")")),
                State::Eq => return Err(String::from("arithmetic syntax error: invalid arithmetic operator (error token is \"=\")"))
            }
            continue;
        }
        if matches!(state, State::Close){
            return Err(String::from("arithmetic syntax error: unexpected token \")\""));
        }
        match parsing(&c, optype, &mut value, &mut result, &mut state) {
            Ok(x) => optype = x,
            Err(e) => return Err(e)
        }
    }
    Err(String::from("unclosed parentheses: (( opened but never closed"))
}

fn arithpart(s_chars: &mut Chars) -> Result<ValueTypes,String>{
    let mut optype = Null;
    let mut state = State::Normal;
    let mut value = String::new();
    let mut result = Operation::None;
    while let Some(c) = s_chars.next() {
        if c == '('{
            let bracket = arithpart(s_chars);
            match bracket {
                Ok(x) => merge_op(&mut optype, x, &mut result, &mut state),
                Err(e) => return Err(e),
            }
        }
        if c == ')'{
            match state {
                State::Normal => {
                    match push_op(&mut optype, &mut value, &mut result, &mut state) {
                        Ok(_) => return Ok(ValueTypes::Cal(result)),
                        Err(e) => return Err(e)
                    }
                },
                State::Not => return Err(String::from("arithmetic syntax error: invalid arithmetic operator (error token is \"!\")")),
                State::Eq => return Err(String::from("arithmetic syntax error: invalid arithmetic operator (error token is \"=\")")),
                State::Close => panic!()
            }
        }
        if matches!(state, State::Close){
            return Err(String::from("arithmetic syntax error: unexpected token \")\""));
        }
        match parsing(&c, optype, &mut value, &mut result, &mut state) {
            Ok(x) => optype = x,
            Err(e) => return Err(e)
        }
    }
    Err(String::from("unclosed parentheses: ( opened but never closed"))
}

fn parsing(c: &char, mut optype: OpType, value: &mut String, result: &mut Operation, state: &mut State) -> Result<OpType, String>{
    if !value.is_empty() && OPERATORS.contains(c) {
        match push_op(&mut optype, value, result, state) {
            Ok(_) => (),
            Err(e) => return Err(e)
        }
    };
    let invalid_op_err = Err(format!("arithmetic syntax error: invalid arithmetic operator (error token is \"{}\")",c));
    match c {
        '&' => match optype {
            BitAnd => optype = LogicAnd,
            Null => optype = BitAnd,
            _ => return invalid_op_err
        },
        '|' => match optype {
            BitOr => optype = LogicOr,
            Null => optype = BitOr,
            _ => return invalid_op_err
        },
        '^' => match optype {
            Null => optype = BitXor,
            _ => return invalid_op_err
        },
        '+' => match optype {
            Null => optype = Add,
            _ => return invalid_op_err
        },
        '-' => match optype {
            Null => optype = Sub,
            _ => return invalid_op_err
        },
        '*' => match optype {
            Mult => optype = Pow,
            Null => optype = Mult,
            _ => return invalid_op_err
        },
        '/' => match optype {
            Null => optype = Div,
            _ => return invalid_op_err
        },
        '%' => match optype {
            Null => optype = Mod,
            _ => return invalid_op_err
        },
        '!' => *state = State::Not,
        '=' => match state {
            State::Normal => {
                match optype {
                    Null => *state = State::Eq,
                    Greater => {
                        *state = State::Normal;
                        optype = GreaterEq;
                    },
                    Less => {
                        *state = State::Normal;
                        optype = LessEq;
                    }
                    _ => return invalid_op_err
                }
            },
            State::Not => {
                *state = State::Normal;
                optype = NotEq;
            },
            State::Eq => {
                *state = State::Normal;
                optype = Eq;
            },
            State::Close => panic!(),
        },
        '>' => match optype {
            Null => optype = Greater,
            _ => return invalid_op_err
        },
        '<' => match optype {
            Null => optype = Less,
            _ => return invalid_op_err
        },
        _ => value.push(*c),
    }
    Ok(optype)
}

fn push_op(optype: &mut OpType, value: &mut String, result: &mut Operation, state: &mut State) -> Result<(), String>{
    let valtype;
    if matches!(state, State::Not){
        valtype = ValueTypes::not_from(&value);
    }else{
        valtype = ValueTypes::from(&value);
    }
    *value = String::new();
    if let Err(e) = valtype{
        return Err(e);
    }
    result.push(valtype.unwrap(), optype.clone());
    *optype = Null;
    Ok(())
}

fn merge_op(optype: &mut OpType, mut value: ValueTypes, result: &mut Operation, state: &mut State){
    if matches!(state, State::Not){
        value = ValueTypes::Cal(Operation::Not(Box::from(value)));
    }
    result.push(value, optype.clone());
    *optype = Null;
}
