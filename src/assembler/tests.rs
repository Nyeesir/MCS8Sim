use crate::assembler;

use super::*;

#[test]
fn assembler_split_expression_test_1(){
    let data = "7 * 2  /13 SHL 2 NOT 03H";
    let split_data = Assembler::split_expression(data);

    assert_eq!(split_data, vec!["7", "*", "2", "/", "13", "SHL", "2", "NOT", "03H"]);
}

#[test]
fn assembler_split_expression_test_2(){
    let data = "  7  ";
    let split_data = Assembler::split_expression(data);
    assert_eq!(split_data, vec!["7"]);
}

#[test]
fn assembler_split_expression_test_3(){
    let data = "(7 * 2)  /13 SHL 2 * 03H";
    let split_data = Assembler::split_expression(data);

    assert_eq!(split_data, vec!["(", "7", "*", "2", ")" , "/", "13", "SHL", "2", "*", "03H"]);
}

#[test]
fn assembler_infix_to_postfix_test1(){
    let data = "34+40/2";
    let split_data = Assembler::split_expression(data);
    let infix_data = Assembler::convert_infix_expr_to_postfix_expr(split_data).unwrap();

    assert_eq!(infix_data, vec!["34","40","2","/","+"])
}
#[test]
fn assembler_infix_to_postfix_test2(){
    let data = "(7 * 2)  /13 SHL 2 * 05";
    let split_data = Assembler::split_expression(data);
    let infix_data = Assembler::convert_infix_expr_to_postfix_expr(split_data).unwrap();
    
    assert_eq!(infix_data, vec!["7","2","*","13","/","2","SHL","05","*"])
}
#[test]
fn assembler_infix_to_postfix_test3(){
    let data = "((10 + 2) *13) + 7 MOD 2";
    let split_data = Assembler::split_expression(data);
    let infix_data = Assembler::convert_infix_expr_to_postfix_expr(split_data).unwrap();

    assert_eq!(infix_data, vec!["10","2","+","13","*","7","2","MOD","+"])
}

//TODO: dopisac test z NOT