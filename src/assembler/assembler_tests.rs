use crate::assembler;
use super::*;

#[test]
fn test_simple_add() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("2+3").unwrap(), 5);
}

#[test]
fn test_operator_precedence() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("2+3*4").unwrap(), 14);
}

#[test]
fn test_parentheses() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("(2+3)*4").unwrap(), 20);
}

#[test]
fn test_subtraction() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("10-3-2").unwrap(), 5);
}

#[test]
fn test_multiplication_and_division() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("20/5*3").unwrap(), 12);
}

#[test]
fn test_modulo() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("20 MOD 6").unwrap(), 2);
}

#[test]
fn test_unary_minus_literal() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("-5").unwrap(), 0xFFFB);
}

#[test]
fn test_double_unary_minus() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("--5").unwrap(), 5);
}

#[test]
fn test_unary_minus_with_parentheses() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("-(2+3)").unwrap(), 0xFFFB);
}

#[test]
fn test_unary_minus_in_expression() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("2*-3").unwrap(), 0xFFFA);
}

#[test]
fn test_not_zero() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("NOT 0").unwrap(), 0xFFFF);
}

#[test]
fn test_not_not() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("NOT NOT 1").unwrap(), 1);
}

#[test]
fn test_not_with_and() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("NOT 1 AND 3").unwrap(), 2);
}

#[test]
fn test_not_precedence() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("NOT 1 + 1").unwrap(), 0xFFFD);
    // NOT (1+1) = NOT 2 = 0xFFFD
}

#[test]
fn test_and_or_xor() {
    let assembler = Assembler::new();
    assert_eq!(assembler.clone().calculate_expression("1 OR 2").unwrap(), 3);
    assert_eq!(assembler.clone().calculate_expression("3 AND 1").unwrap(), 1);
    assert_eq!(assembler.clone().calculate_expression("3 XOR 1").unwrap(), 2);
}

#[test]
fn test_shift_left() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("1 SHL 4").unwrap(), 16);
}

#[test]
fn test_shift_right() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("16 SHR 3").unwrap(), 2);
}

#[test]
fn test_complex_expression() {
    let expr = "NOT (2 + 3*4) AND 0FFFFH";
    // 3*4=12, +2=14, NOT 14 = 0xFFF1
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression(expr).unwrap(), 0xFFF1);
}

#[test]
fn test_many_operators() {
    let expr = "1 + 2 SHL 3 AND NOT 0";
    // 2 SHL 3 = 16
    // NOT 0 = 0xFFFF
    // 16 AND 0xFFFF = 16
    // 1 + 16 = 17

    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression(expr).unwrap(), 17);
}

#[test]
fn test_add_overflow() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("65535+1").unwrap(), 0);
}

#[test]
fn test_mul_overflow() {
    let assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("256*256").unwrap(), 0);
}

#[test]
fn test_unbalanced_parentheses() {
    let assembler = Assembler::new();
    assert!(assembler.calculate_expression("(1+2").is_err());
}

#[test]
fn test_invalid_operator_sequence() {
    let assembler = Assembler::new();
    assert!(assembler.calculate_expression("1 + * 2").is_err());
}

#[test]
fn test_missing_operand() {
    let assembler = Assembler::new();
    assert!(assembler.calculate_expression("NOT").is_err());
}

#[test]
fn test_empty_expression() {
    let assembler = Assembler::new();
    assert!(assembler.calculate_expression("").is_err());
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

#[test]
fn parse_8bit_number_valid_positive() {
    assert_eq!(Assembler::parse_8bit_number("0").unwrap(), 0);
    assert_eq!(Assembler::parse_8bit_number("127").unwrap(), 127);
    assert_eq!(Assembler::parse_8bit_number("255").unwrap(), 255);
}

#[test]
fn parse_8bit_number_valid_negative() {
    assert_eq!(Assembler::parse_8bit_number("-1").unwrap(), 255);
    assert_eq!(Assembler::parse_8bit_number("-128").unwrap(), 128);
}

#[test]
fn parse_8bit_number_out_of_range() {
    assert!(Assembler::parse_8bit_number("256").is_err());
    assert!(Assembler::parse_8bit_number("-129").is_err());
}

#[test]
fn parse_8bit_number_invalid_input() {
    assert!(Assembler::parse_8bit_number("abc").is_err());
    assert!(Assembler::parse_8bit_number("").is_err());
}

#[test]
fn parse_16bit_number_valid_positive() {
    assert_eq!(Assembler::parse_16bit_number("0").unwrap(), 0);
    assert_eq!(Assembler::parse_16bit_number("32767").unwrap(), 32767);
    assert_eq!(Assembler::parse_16bit_number("65535").unwrap(), 65535);
}

#[test]
fn parse_16bit_number_valid_negative() {
    assert_eq!(Assembler::parse_16bit_number("-1").unwrap(), 65535);
    assert_eq!(Assembler::parse_16bit_number("-32768").unwrap(), 32768);
}

#[test]
fn parse_16bit_number_out_of_range() {
    assert!(Assembler::parse_16bit_number("65536").is_err());
    assert!(Assembler::parse_16bit_number("-32769").is_err());
}

#[test]
fn parse_16bit_number_invalid_input() {
    assert!(Assembler::parse_16bit_number("xyz").is_err());
    assert!(Assembler::parse_16bit_number("").is_err());
}

#[test]
fn test_here_and_dollar_sign(){
    let assembler = Assembler::new();
    assert_eq!(assembler.clone().calculate_expression("HERE").unwrap(), 0);
    assert_eq!(assembler.clone().calculate_expression("HERE + 3").unwrap(), 3);
    assert_eq!(assembler.clone().calculate_expression("$").unwrap(), 0);
    assert_eq!(assembler.clone().calculate_expression("$ + 1100B").unwrap(), 12);
}

#[test]
#[should_panic]
fn assembler_mvi_test_5() {
    let assembler = Assembler::new();
    let data = "MVI B,256";
    println!("{:?}", Assembler::fetch_fields(data));
    assembler::Assembler::new().assemble(data).unwrap();
}

// #[test]
// fn assemble_file(){
//     use std::fs;
//     let file_path = "src/assembler/test_files/asm_test.asm";
//     let file_content = fs::read_to_string(file_path).unwrap();
//     let mut assembler = Assembler::new();
//     let assembled_code = assembler.assemble(file_content.as_str()).unwrap();
// }