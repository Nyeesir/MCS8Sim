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
    let data = "(7 * 2)  /13 SHL 2 NOT 03H";
    let split_data = Assembler::split_expression(data);

    assert_eq!(split_data, vec!["(", "7", "*", "2", ")" , "/", "13", "SHL", "2", "NOT", "03H"]);
}