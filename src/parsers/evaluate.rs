use std::str::Chars;
use crate::types::{ArgPart, OpType, Operation, ValueTypes};
use crate::types::OpType::*;

const OPERATORS: [char;12] = ['&','|','^','=','!','>','<','+','-','*','/','%'];

enum State{
    Close, Not, Normal
}

pub fn arithexp(s_chars: &mut Chars) -> Result<ArgPart, String>{
    let mut optype = Null;
    let mut state = State::Normal;
    let mut value = String::new();
    let mut result = Operation::None;
    while let Some(c) = s_chars.next() {
        if c == ')'{
            match state {
                State::Close => {
                    match push_op(&mut optype, &mut value, &mut result) {
                        Ok(_) => return Ok(ArgPart::ArithEx(result)),
                        Err(e) => return Err(e)
                    }
                },
                State::Normal => state = State::Close,
                State::Not => return Err(String::from("arithmetic syntax error: invalid arithmetic operator (error token is \"!\")"))
            }
            continue;
        }
        match parsing(&c, optype, &mut value, &mut result) {
            Ok(x) => optype = x,
            Err(e) => return Err(e)
        }
    }
    Err(String::from("unclosed parentheses: (( opened but never closed"))
}

fn parsing(c: &char, mut optype: OpType, value: &mut String, result: &mut Operation) -> Result<OpType, String>{
    if !value.is_empty() && OPERATORS.contains(c) {
        match push_op(&mut optype, value, result) {
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
        _ => value.push(*c),
    }
    Ok(optype)
}

fn push_op(optype: &mut OpType,value: &mut String, result: &mut Operation) -> Result<(), String>{
    let valtype = ValueTypes::from(&value);
    *value = String::new();
    if let Err(e) = valtype{
        return Err(e);
    }
    result.push(valtype.unwrap(), optype.clone());
    *optype = Null;
    Ok(())
}
