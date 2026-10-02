use std::rc::Rc;
use std::cell::RefCell;
use crate::BANNED;

#[derive(Clone)]
pub enum ValueTypes{
    Var(String),
    Literal(String),
    Int(i32),
    Float(f32),
    Boolean(bool),
    Cal(Operation)
}

impl ValueTypes {
    pub fn from(s: &String) -> Result<ValueTypes,String>{
        let s_trim = String::from(s.trim());
        if s_trim.len() > 2 && s_trim.starts_with('"') && s_trim.ends_with('"'){
            let inner = String::from(&s_trim[1..s_trim.len()-1]);
            return Ok(ValueTypes::Literal(inner))
        }
        if let Ok(x) = s_trim.parse::<i32>(){
            return Ok(ValueTypes::Int(x))
        }
        if let Ok(x) = s_trim.parse::<f32>(){
            return Ok(ValueTypes::Float(x))
        }
        if let Ok(x) = s_trim.parse::<bool>(){
            return Ok(ValueTypes::Boolean(x))
        }
        if let Some(x) = s_trim.chars().find(|x| BANNED.contains(x)){
            return Err(format!("invalid char: {}",x));
        }
        Ok(ValueTypes::Var(s_trim))
    }
    pub fn not_from(s: &String) -> Result<ValueTypes,String>{
        match Self::from(s) {
            Ok(x) => Ok(Self::Cal(Operation::Not(Box::from(x)))),
            Err(e) => Err(e)
        }
    }
    pub fn to_string(&self) -> String{
        use ValueTypes::*;
        match self {
            Var(x) => format!("Var({})",x),
            Literal(x) => String::from(x),
            Int(x) => format!("Int({})",x),
            Float(x) => format!("Float({})",x),
            Boolean(x) => format!("Boolean({})",x),
            Cal(x) => x.to_string()
        }
    }
}

#[derive(Clone,PartialEq)]
pub enum OpType{
    LogicAnd, LogicOr,
    BitAnd, BitOr, BitXor,
    Eq, NotEq, Greater, GreaterEq, Less, LessEq,
    Add, Sub, Mult, Div, Pow, Mod,
    Null
}

impl OpType{
    pub fn get_priority(&self) -> u8{
        use OpType::*;
        match self {
            //lower value = higher priority
            Pow => 0,
            Mod | Mult | Div | BitAnd => 1,
            Add | Sub | BitOr | BitXor => 2,
            LogicAnd | LogicOr => 4,
            _ => 3
        }
    }
    pub fn to_string(&self) -> String {
        use OpType::*;
        match self {
            LogicAnd => String::from("And"),
            LogicOr => String::from("Or"),
            BitAnd => String::from("BitAnd"),
            BitOr => String::from("BitOr"),
            BitXor => String::from("BitXor"),
            Eq => String::from("Eq"),
            NotEq => String::from("NotEq"),
            Greater => String::from("Greater"),
            GreaterEq => String::from("GreaterEq"),
            Less => String::from("Less"),
            LessEq => String::from("LessEq"),
            Add => String::from("Add"),
            Sub => String::from("Sub"),
            Mult => String::from("Mult"),
            Div => String::from("Div"),
            Pow => String::from("Pow"),
            Mod => String::from("Mod"),
            Null => String::new()
        }
    }
}

#[derive(Clone)]
pub enum Operation{
    Op(OpType, Vec<ValueTypes>),
    Not(Box<ValueTypes>),
    Val(Box<ValueTypes>),
    None
}

impl Operation{
    pub fn from(s: String) -> Result<Operation,String>{
        match ValueTypes::from(&s) {
            Ok(x) => Ok(Operation::Val(Box::new(x))),
            Err(e) => Err(e)
        }
    }
    pub fn to_string(&self) -> String{
        use Operation::*;
        match self {
            Not(x) => format!("Not({})",x.to_string()),
            Val(x) => format!("{}",x.to_string()),
            Op(t,v) => format!("{}[{}]",t.to_string(),v.into_iter().map(|a| a.to_string()).collect::<Vec<String>>().join(",")),
            None => String::new()
        }
    }
    fn val_from(val: ValueTypes) -> Operation{
        Self::Val(Box::new(val))
    }
    fn merge(left:Vec<ValueTypes>, op_old: OpType, right:ValueTypes, op_new: OpType) -> Operation{
        use Operation::*;

        if op_new == op_old{
            let mut new_left = left;
            new_left.push(right);
            return Op(op_new,new_left);
        }

        if op_old.get_priority() <= op_new.get_priority(){
            let new_left = ValueTypes::Cal(Op(op_old,left));
            return Op(op_new,vec![new_left,right]);
        }

        let mut new_left = left;
        let left_last = new_left.pop().unwrap();
        let new_right = ValueTypes::Cal(Op(op_new,vec![left_last,right]));
        new_left.push(new_right);
        Op(op_old,new_left)
    }
    pub fn push(&mut self, val: ValueTypes, optype: OpType){
        use Operation::*;

        let old_self = std::mem::replace(self,None);
        match old_self {
            Not(_) => *self = Op(optype,vec![ValueTypes::Cal(old_self),val]),
            Val(x) => *self = Op(optype,vec![*x,val]),
            Op(op, v) => *self = Self::merge(v, op, val, optype),
            None => *self = Val(Box::new(val))
        }
    }
}

#[derive(Clone)]
pub enum ArgPart{
    Literal(String),
    Var(String),
    ArithEx(Operation),
    ComSub(ActionList),
}

impl ArgPart{
    fn to_string(&self) -> String{
        use ArgPart::*;
        match self {
            Literal(x) => String::from(x),
            Var(x) => format!("Var({})",x),
            ComSub(x) => format!("ComSub({})",x.display()),
            ArithEx(x) => format!("ArithEx({})",x.to_string())
        }
    }
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

impl Action{
    fn to_string(&self) -> String{
        use Action::*;
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
    com: bool,
}

impl ActionList{
    pub fn new() -> Self{
        let new_node = Rc::new(RefCell::new(ActionNode {
            action: None,
            next: None
        }));
        ActionList { head: new_node.clone(), tail: new_node, length: 0 , com: false}
    }
    pub fn push(&mut self, action: Action){
        if matches!(action, Action::Command(_)){
            self.com = true;
        }
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
    pub fn push_none(&mut self){
        let new_node = Rc::new(RefCell::new(ActionNode {
            action: None,
            next: None
        }));
        self.tail.borrow_mut().next = Some(new_node.clone());
        self.tail = new_node;
        self.length += 1;
    }
    pub fn set_tail(&mut self, action: Action){
        if matches!(action, Action::Command(_)){
            self.com = true;
        }
        if self.len() == 0{
            self.length = 1;
        }
        self.tail.borrow_mut().action = Some(action);
    }
    pub fn push_arg(&mut self, arg: ArgPart){
        use Action::Arg;
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
    pub fn has_com(&self) -> bool{
        self.com
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
mod actionlist{
    use super::*;
    use super::ArgPart::*;

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
        use Action::*;

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
        c.push(Background);
        assert_eq!("Pipe(Command(mkdir) Arg[dir,Var(var1)] Arg[dir2]) Command(grep) Arg[Var(var2)] Background",c.display());
    }

    #[test]
    fn next(){
        let mut a = ActionList::new();
        a.push(Action::Command(String::from("mkdir")));
        a.push_arg(Literal(String::from("dir")));
        a.push_arg(Var(String::from("var1")));
        a.push(Action::Arg(Vec::new()));
        a.push_arg(Literal(String::from("dir2")));
        assert_eq!("Command(mkdir)",a.next().unwrap().to_string());
        assert_eq!("Arg[dir,Var(var1)] Arg[dir2]",a.display());
        assert_eq!("Arg[dir,Var(var1)]",a.next().unwrap().to_string());
        a.next();
        assert!(a.next().is_none())
    }
}

#[cfg(test)]
mod operation{

    use super::*;

    #[test]
    fn from(){
        let a = Operation::from(String::from("var"));
        assert_eq!("Var(var)",a.unwrap().to_string());
        let b = Operation::from(String::from("\"Some text\""));
        assert_eq!("Some text",b.unwrap().to_string());
        let c = Operation::from(String::from("69"));
        assert_eq!("Int(69)",c.unwrap().to_string());
        let d = Operation::from(String::from("69.67"));
        assert_eq!("Float(69.67)",d.unwrap().to_string());
        let e = Operation::from(String::from("true"));
        assert_eq!("Boolean(true)",e.unwrap().to_string());
        let f = Operation::from(String::from("false"));
        assert_eq!("Boolean(false)",f.unwrap().to_string());

        let g = Operation::from(String::from("var\\"));
        assert_eq!("invalid char: \\",g.err().unwrap());
        let h = Operation::from(String::from("some\"thing"));
        assert_eq!("invalid char: \"",h.err().unwrap());
    }

    #[test]
    fn push(){
        use OpType::*;
        use ValueTypes::*;
        let mut a = Operation::from(String::from("a")).unwrap();
        // a && b
        a.push(Var(String::from("b")),BitAnd);
        // a && b
        assert_eq!("BitAnd[Var(a),Var(b)]",a.to_string());
        // a && b + 12
        a.push(Int(12),Add);
        // (a && b) + 12
        assert_eq!("Add[BitAnd[Var(a),Var(b)],Int(12)]",a.to_string());
        // a && b + 12 * 69
        a.push(Int(69),Mult);
        // (a && b) + (12 * 69)
        assert_eq!("Add[BitAnd[Var(a),Var(b)],Mult[Int(12),Int(69)]]",a.to_string());

        let mut b = Operation::from(String::from("a")).unwrap();
        // a - 69.420
        b.push(Float(69.420), Sub);
        // a - 69.420
        assert_eq!("Sub[Var(a),Float(69.42)]",b.to_string());
        // a && b + 12 * 69 / (a - 69.420)
        a.push(Cal(b), Div);
        // (a && b) + ((12 * 69) / (a - 69.420))
        assert_eq!("Add[BitAnd[Var(a),Var(b)],Div[Mult[Int(12),Int(69)],Sub[Var(a),Float(69.42)]]]",a.to_string());

        let mut c = Operation::from(String::from("9")).unwrap();
        c.push(Int(10), Add);
        c.push(Int(21), Add);
        assert_eq!("Add[Int(9),Int(10),Int(21)]",c.to_string());
    }
}
