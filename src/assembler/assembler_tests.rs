use crate::assembler;

use super::*;

#[test]
fn test_simple_add() {
    assert_eq!(Assembler::calculate_expression("2+3").unwrap(), 5);
}

#[test]
fn test_operator_precedence() {
    assert_eq!(Assembler::calculate_expression("2+3*4").unwrap(), 14);
}

#[test]
fn test_parentheses() {
    assert_eq!(Assembler::calculate_expression("(2+3)*4").unwrap(), 20);
}

#[test]
fn test_subtraction() {
    assert_eq!(Assembler::calculate_expression("10-3-2").unwrap(), 5);
}

#[test]
fn test_multiplication_and_division() {
    assert_eq!(Assembler::calculate_expression("20/5*3").unwrap(), 12);
}

#[test]
fn test_modulo() {
    assert_eq!(Assembler::calculate_expression("20 MOD 6").unwrap(), 2);
}

#[test]
fn test_unary_minus_literal() {
    assert_eq!(Assembler::calculate_expression("-5").unwrap(), 0xFFFB);
}

#[test]
fn test_double_unary_minus() {
    assert_eq!(Assembler::calculate_expression("--5").unwrap(), 5);
}

#[test]
fn test_unary_minus_with_parentheses() {
    assert_eq!(Assembler::calculate_expression("-(2+3)").unwrap(), 0xFFFB);
}

#[test]
fn test_unary_minus_in_expression() {
    assert_eq!(Assembler::calculate_expression("2*-3").unwrap(), 0xFFFA);
}

#[test]
fn test_not_zero() {
    assert_eq!(Assembler::calculate_expression("NOT 0").unwrap(), 0xFFFF);
}

#[test]
fn test_not_not() {
    assert_eq!(Assembler::calculate_expression("NOT NOT 1").unwrap(), 1);
}

#[test]
fn test_not_with_and() {
    assert_eq!(Assembler::calculate_expression("NOT 1 AND 3").unwrap(), 2);
}

#[test]
fn test_not_precedence() {
    assert_eq!(Assembler::calculate_expression("NOT 1 + 1").unwrap(), 0xFFFD);
    // NOT (1+1) = NOT 2 = 0xFFFD
}

#[test]
fn test_and_or_xor() {
    assert_eq!(Assembler::calculate_expression("1 OR 2").unwrap(), 3);
    assert_eq!(Assembler::calculate_expression("3 AND 1").unwrap(), 1);
    assert_eq!(Assembler::calculate_expression("3 XOR 1").unwrap(), 2);
}

#[test]
fn test_shift_left() {
    assert_eq!(Assembler::calculate_expression("1 SHL 4").unwrap(), 16);
}

#[test]
fn test_shift_right() {
    assert_eq!(Assembler::calculate_expression("16 SHR 3").unwrap(), 2);
}

#[test]
fn test_complex_expression() {
    let expr = "NOT (2 + 3*4) AND 0FFFFH";
    // 3*4=12, +2=14, NOT 14 = 0xFFF1
    assert_eq!(Assembler::calculate_expression(expr).unwrap(), 0xFFF1);
}

#[test]
fn test_many_operators() {
    let expr = "1 + 2 SHL 3 AND NOT 0";
    // 2 SHL 3 = 16
    // NOT 0 = 0xFFFF
    // 16 AND 0xFFFF = 16
    // 1 + 16 = 17
    assert_eq!(Assembler::calculate_expression(expr).unwrap(), 17);
}

#[test]
fn test_add_overflow() {
    assert_eq!(Assembler::calculate_expression("65535+1").unwrap(), 0);
}

#[test]
fn test_mul_overflow() {
    assert_eq!(Assembler::calculate_expression("256*256").unwrap(), 0);
}

#[test]
fn test_unbalanced_parentheses() {
    assert!(Assembler::calculate_expression("(1+2").is_err());
}

#[test]
fn test_invalid_operator_sequence() {
    assert!(Assembler::calculate_expression("1 + * 2").is_err());
}

#[test]
fn test_missing_operand() {
    assert!(Assembler::calculate_expression("NOT").is_err());
}

#[test]
fn test_empty_expression() {
    assert!(Assembler::calculate_expression("").is_err());
}

#[test]
fn parse_octal_o() {
    assert_eq!(Assembler::parse_number_i32("56O").unwrap(), 46);
}

#[test]
fn parse_octal_q() {
    assert_eq!(Assembler::parse_number_i32("566q").unwrap(), 374);
}

#[test]
fn parse_binary() {
    assert_eq!(Assembler::parse_number_i32("1110011b").unwrap(), 115);
}

#[test]
fn parse_hexadecimal() {
    assert_eq!(Assembler::parse_number_i32("06AH").unwrap(), 106);
}

#[test]
#[should_panic]
fn parse_hexadecimal_panic() {
    assert_eq!(Assembler::parse_number_i32("AAH").unwrap(), 170);
}

#[test]
fn parse_decimal_d() {
    assert_eq!(Assembler::parse_number_i32("0037D").unwrap(), 37);
}

#[test]
fn parse_decimal() {
    assert_eq!(Assembler::parse_number_i32("145").unwrap(), 145);
}

#[test]
#[should_panic]
fn parse_number_blank() {
    assert_eq!(Assembler::parse_number_i32("").unwrap(), 0);
}
