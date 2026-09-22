use std::rc::Rc;
use std::cell::RefCell;

use crate::types::Action::{And, Arg, Background, Command, Or, Pipe, Sepr};
use crate::types::ArgPart::{Literal,Var};

#[derive(Clone)]
pub struct Condition{

}

#[derive(Clone)]
pub struct Calculate{

}

#[derive(Clone)]
pub enum ArgPart{
    Literal(String),
    Var(String),
    Test(Condition),
    ArithEx(Calculate),
    ComSub(ActionList),
}

#[derive(Clone)]
pub enum Action{
    Command(String),
    Arg(Vec<ArgPart>),
    And(ActionList),
    Or(ActionList),
    Sepr(ActionList),
    Pipe(ActionList),
    Background
}

#[derive(Clone)]
pub struct ActionNode{
    action: Option<Action>,
    next: Option<Rc<RefCell<ActionNode>>>
}

#[derive(Clone)]
pub struct ActionList{
    head: Rc<RefCell<ActionNode>>,
    tail: Rc<RefCell<ActionNode>>,
    length: i32,
}

impl Action{
    fn to_string(&self) -> String{
        match self {
            Command(x) => format!("Command({})",x),
            Arg(x) => {
                let mut s = String::from("Arg[");
                for argpart in x{
                    s += &argpart.to_string();
                    s.push(',');
                }
                s.pop();
                s.push(']');
                s
            },
            And(x) => format!("And({})",x.display()),
            Or(x) => format!("Or({})",x.display()),
            Sepr(x) => format!("Sepr({})",x.display()),
            Pipe(x) => format!("Pipe({})",x.display()),
            Background => String::from("Background")
        }
    }
}

impl ArgPart{
    fn to_string(&self) -> String{
        match self {
            Literal(x) => String::from(x),
            Var(x) => format!("Var({})",x),
            _ => todo!()
        }
    }
}

impl ActionList{
    pub fn new() -> Self{
        let new_node = Rc::new(RefCell::new(ActionNode {
            action: None,
            next: None
        }));
        ActionList { head: new_node.clone(), tail: new_node, length: 0 }
    }
    pub fn push(&mut self, action: Action){
        let new_node = Rc::new(RefCell::new(ActionNode {
            action: Some(action),
            next: None
        }));
        self.tail.borrow_mut().next = Some(new_node.clone());
        self.tail = new_node;
        if self.length == 0{
            self.next();
        }
        self.length += 1;
    }
    pub fn set_tail(&mut self, action: Action){
        self.tail.borrow_mut().action = Some(action);
    }
    pub fn push_arg(&mut self, arg: ArgPart){
        let mut tail_ref = self.tail.borrow_mut();
        match &mut tail_ref.action {
            Some(Arg(x)) => {
                x.push(arg);
            },
            Some(_) => {
                drop(tail_ref);
                self.push(Arg(vec![arg]));
            }
            None => {
                drop(tail_ref);
                self.set_tail(Arg(vec![arg]));
            }
        }
    }
    pub fn len(&self) -> i32{
        self.length
    }
    pub fn next(&mut self) -> Option<Action>{
        let action = self.head.borrow().action.clone();
        if self.len() == 1{
            panic!("");
        }
        let next = self.head.borrow().next.clone();
        if next.is_none(){
            return None;
        }
        self.head = next.unwrap();
        action
    }
    pub fn display(&self) -> String{
        let mut s = String::new();
        let mut cur = Some(self.head.clone());
        while let Some(node_rc) = cur {
            let node = node_rc.borrow();
            if let Some(action) = node.clone().action{
                s += &action.to_string();
            }
            cur = node.next.clone();
            s.push(' ');
        }
        s.pop();
        s
    }
    pub fn and(old: ActionList) -> ActionList{
        let mut new = ActionList::new();
        new.push(Action::And(old));
        new
    }
    pub fn or(old: ActionList) -> ActionList{
        let mut new = ActionList::new();
        new.push(Action::Or(old));
        new
    }
    pub fn sepr(old: ActionList) -> ActionList{
        let mut new = ActionList::new();
        new.push(Action::Sepr(old));
        new
    }
    pub fn pipe(old: ActionList) -> ActionList{
        let mut new = ActionList::new();
        new.push(Action::Pipe(old));
        new
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn command(){
        let mut a = ActionList::new();
        a.push(Action::Command(String::from("cd")));
        assert_eq!(String::from("Command(cd)"),a.display());
    }

    #[test]
    fn arg(){
        let mut a = ActionList::new();
        a.push(Action::Command(String::from("mkdir")));
        a.push_arg(Literal(String::from("dir")));
        assert_eq!("Command(mkdir) Arg[dir]",a.display());

        a.push_arg(Var(String::from("var1")));
        assert_eq!("Command(mkdir) Arg[dir,Var(var1)]",a.display());

        a.push_arg(Literal(String::from("1")));
        assert_eq!("Command(mkdir) Arg[dir,Var(var1),1]",a.display());

        a.push(Action::Arg(Vec::new()));
        a.push_arg(Literal(String::from("dir2")));
        assert_eq!("Command(mkdir) Arg[dir,Var(var1),1] Arg[dir2]",a.display());
    }

    #[test]
    fn other_action(){
        let mut base = ActionList::new();
        base.push(Action::Command(String::from("mkdir")));
        base.push_arg(Literal(String::from("dir")));
        base.push_arg(Var(String::from("var1")));
        base.push(Action::Arg(Vec::new()));
        base.push_arg(Literal(String::from("dir2")));
        let mut a = base.clone();
        a = ActionList::and(a);
        a.push(Command(String::from("cd")));
        assert_eq!("And(Command(mkdir) Arg[dir,Var(var1)] Arg[dir2]) Command(cd)",a.display());

        let mut b = base.clone();
        b = ActionList::or(b);
        b.push(Command(String::from("ls")));
        assert_eq!("Or(Command(mkdir) Arg[dir,Var(var1)] Arg[dir2]) Command(ls)",b.display());

        a = ActionList::sepr(a);
        a.push(Command(String::from("fastfetch")));
        assert_eq!("Sepr(And(Command(mkdir) Arg[dir,Var(var1)] Arg[dir2]) Command(cd)) Command(fastfetch)",a.display());

        let mut c = base;
        c = ActionList::pipe(c);
        c.push(Command(String::from("grep")));
        c.push_arg(Var(String::from("var2")));
        assert_eq!("Pipe(Command(mkdir) Arg[dir,Var(var1)] Arg[dir2]) Command(grep) Arg[Var(var2)]",c.display());
    }
}
