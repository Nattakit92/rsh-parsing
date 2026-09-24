use crate::types::{Action, ActionList, ArgPart};

fn push_str(s: String, result: &mut ActionList){
    if result.has_com(){
        result.push_arg(ArgPart::Literal(s));
    }else{
        result.set_tail(Action::Command(s));
    }
}

pub fn parsing(s: String) -> Result<ActionList,String>{
    let mut result = ActionList::new();
    let mut s_chars = s.chars();
    let mut str_slice = String::new();
    while let Some(c) = s_chars.next() {
        match c {
            ' ' => {
                push_str(str_slice.clone(), &mut result);
                str_slice = String::new();
                result.push_none();
            },
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
    }
}
