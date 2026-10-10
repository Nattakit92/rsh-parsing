#[cfg(test)]
mod api_test{
    use rsh_parser;
    use rsh_parser::types::*;

    //simple tokenizer
    #[test]
    fn case1(){
        let mut a = rsh_parser::parse("command").unwrap();
        assert_eq!(Action::Command(String::from("command")),a.next().unwrap());
        let mut b = rsh_parser::parse("com arg").unwrap();
        assert_eq!(Action::Command(String::from("com")),b.next().unwrap());
        assert_eq!(Action::Arg(vec![ArgPart::Literal(String::from("arg"))]),b.next().unwrap());
        let mut c = rsh_parser::parse("com arg1 arg2").unwrap();
        c.next();
        assert_eq!(Action::Arg(vec![ArgPart::Literal(String::from("arg1"))]),c.next().unwrap());
        assert_eq!(Action::Arg(vec![ArgPart::Literal(String::from("arg2"))]),c.next().unwrap());
    }

    //variable
    #[test]
    fn case2(){
        let mut a = rsh_parser::parse("command arg$var").unwrap();
        a.next();
        assert_eq!(Action::Arg(vec![ArgPart::Literal(String::from("arg")),ArgPart::Var(String::from("var"))]),a.next().unwrap());
        let mut b = rsh_parser::parse("com ${var1}${var2}").unwrap();
        b.next();
        assert_eq!(Action::Arg(vec![ArgPart::Var(String::from("var1")),ArgPart::Var(String::from("var2"))]),b.next().unwrap());
    }
}
