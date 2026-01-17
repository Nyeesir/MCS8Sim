use super::*;

#[test]
fn test_range() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("0FFFFH", 0, true).unwrap(), Some(0xFFFF));
}

#[test]
fn test_range_minus() {
    let mut assembler = Assembler::new();
    let val = (i16::MIN).to_string();
    assert_eq!(assembler.calculate_expression(&val,0, true).unwrap(), Some(-0x8000));
}

#[test]
fn test_8bit_range() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.parse_8bit_expr("0FFH",0).unwrap(), 0xFF);
}

#[test]
fn test_8bit_range_minus() {
    let mut assembler = Assembler::new();
    let val = (i8::MIN).to_string();
    assert_eq!(assembler.parse_8bit_expr(&val,0).unwrap(), 0x80);
}

#[test]
fn test_16bit_range() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.parse_16bit_expr("0FFFFH",0).unwrap(), (0xFF,0xFF));
}

#[test]
fn test_16bit_range_minus() {
    let mut assembler = Assembler::new();
    let val = (i16::MIN).to_string();
    //adresy wiec le i jest na odwrot
    assert_eq!(assembler.parse_16bit_expr(&val,0).unwrap(), (0x00,0x80));
}

#[test]
fn test_simple_add() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("2+3",0, true).unwrap(), Some(5));
}

#[test]
fn test_hex() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("0A3H",0, true).unwrap(), Some(0xA3));
}

#[test]
fn test_minus_hex() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.parse_8bit_expr("-03H",0).unwrap(), 0xFD);
}

#[test]
fn test_operator_precedence() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("2+3*4",0, true).unwrap(), Some(14));
}

#[test]
fn test_parentheses() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("(2+3)*4",0, true).unwrap(), Some(20));
}

#[test]
fn test_subtraction() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("10-3-2",0, true).unwrap(), Some(5));
}

#[test]
fn test_multiplication_and_division() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("20/5*3",0, true).unwrap(), Some(12));
}

#[test]
fn test_modulo() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("20 MOD 6",0, true).unwrap(), Some(2));
}

#[test]
fn test_unary_minus_literal() {
    let mut assembler = Assembler::new();
    //le
    assert_eq!(assembler.parse_16bit_expr("-5",0).unwrap(), 0xFFFB_u16.to_le_bytes().into());
}

#[test]
fn test_double_unary_minus() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("--5",0, true).unwrap(), Some(5));
}

#[test]
fn test_unary_minus_with_parentheses() {
    let mut assembler = Assembler::new();
    //le
    assert_eq!(assembler.parse_16bit_expr("-(2+3)",0).unwrap(),  0xFFFB_u16.to_le_bytes().into());
}

#[test]
fn test_subtraction_2(){
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("2FH - 0AH",0, true).unwrap(), Some(0x25));
}

#[test]
fn test_unary_minus_in_expression() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("2*-3",0, true).unwrap(), Some(0xFFFA));
}

#[test]
fn test_not_zero() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("NOT 0",0, true).unwrap(), Some(0xFFFF));
}

#[test]
fn test_not_not() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("NOT NOT 1",0, true).unwrap(), Some(1));
}

#[test]
fn test_not_with_and() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("NOT 1 AND 3",0, true).unwrap(), Some(2));
}

#[test]
fn test_not_precedence() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("NOT 1 + 1",0, true).unwrap(), Some(0xFFFD));
    // NOT (1+1) = NOT 2 = 0xFFFD
}

#[test]
fn test_and_or_xor() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("1 OR 2",0, true).unwrap(), Some(3));
    assert_eq!(assembler.calculate_expression("3 AND 1",0, true).unwrap(), Some(1));
    assert_eq!(assembler.calculate_expression("3 XOR 1",0, true).unwrap(), Some(2));
}

#[test]
fn test_shift_left() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("1 SHL 4",0, true).unwrap(), Some(16));
}

#[test]
fn test_shift_right() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("16 SHR 3",0, true).unwrap(), Some(2));
}

#[test]
fn test_complex_expression() {
    let expr = "NOT (2 + 3*4) AND 0FFFFH";
    // 3*4=12, +2=14, NOT 14 = 0xFFF1
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression(expr,0, true).unwrap(), Some(0xFFF1));
}

#[test]
fn test_many_operators() {
    let expr = "1 + 2 SHL 3 AND NOT 0";
    // 2 SHL 3 = 16
    // NOT 0 = 0xFFFF
    // 16 AND 0xFFFF = 16
    // 1 + 16 = 17

    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression(expr,0, true).unwrap(), Some(17));
}

#[test]
fn test_add_overflow() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("65535+1",0, true).unwrap(), Some(0));
}

#[test]
fn test_mul_overflow() {
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("256*256",0, true).unwrap(), Some(0));
}

#[test]
fn test_unbalanced_parentheses() {
    let mut assembler = Assembler::new();
    assert!(assembler.calculate_expression("(1+2",0, true).is_err());
}

#[test]
fn test_invalid_operator_sequence() {
    let mut assembler = Assembler::new();
    assert!(assembler.calculate_expression("1 + * 2",0, true).is_err());
}

#[test]
fn test_empty_expression() {
    let mut assembler = Assembler::new();
    assert!(assembler.calculate_expression("",0, true).is_err());
}

#[test]
fn test_missing_operand() {
    let mut assembler = Assembler::new();
    assert!(assembler.calculate_expression("NOT",0, true).is_err());
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

// #[test]
// fn parse_8bit_number_valid_positive() {
//     assert_eq!(Assembler::parse_8bit_number("0").unwrap(), 0);
//     assert_eq!(Assembler::parse_8bit_number("127").unwrap(), 127);
//     assert_eq!(Assembler::parse_8bit_number("255").unwrap(), 255);
// }
//
// #[test]
// fn parse_8bit_number_valid_negative() {
//     assert_eq!(Assembler::parse_8bit_number("-1").unwrap(), 255);
//     assert_eq!(Assembler::parse_8bit_number("-128").unwrap(), 128);
// }
//
// #[test]
// fn parse_8bit_number_out_of_range() {
//     assert!(Assembler::parse_8bit_number("256").is_err());
//     assert!(Assembler::parse_8bit_number("-129").is_err());
// }
//
// #[test]
// fn parse_8bit_number_invalid_input() {
//     assert!(Assembler::parse_8bit_number("abc").is_err());
//     assert!(Assembler::parse_8bit_number("").is_err());
// }
//
// #[test]
// fn parse_16bit_number_valid_positive() {
//     assert_eq!(Assembler::parse_16bit_number("0").unwrap(), 0);
//     assert_eq!(Assembler::parse_16bit_number("32767").unwrap(), 32767);
//     assert_eq!(Assembler::parse_16bit_number("65535").unwrap(), 65535);
// }
//
// #[test]
// fn parse_16bit_number_valid_negative() {
//     assert_eq!(Assembler::parse_16bit_number("-1").unwrap(), 65535);
//     assert_eq!(Assembler::parse_16bit_number("-32768").unwrap(), 32768);
// }
//
// #[test]
// fn parse_16bit_number_out_of_range() {
//     assert!(Assembler::parse_16bit_number("65536").is_err());
//     assert!(Assembler::parse_16bit_number("-32769").is_err());
// }
//
// #[test]
// fn parse_16bit_number_invalid_input() {
//     assert!(Assembler::parse_16bit_number("xyz").is_err());
//     assert!(Assembler::parse_16bit_number("").is_err());
// }

#[test]
fn db_test() {
    let mut assembler = Assembler::new();
    let operands = vec!["0A3H".to_string()];
    let data = assembler.handle_data_statement("DB", &Some(operands)).unwrap();
    assert_eq!(data, vec![0xA3]);
}

#[test]
fn db_test_multiple_operands() {
    let mut assembler = Assembler::new();
    let operands = vec!["5 * 2".to_string(),"2FH - 0AH".to_string()];
    let data = assembler.handle_data_statement("DB", &Some(operands)).unwrap();
    assert_eq!(data, vec![0x0A,0x25]);
}

#[test]
fn db_test_expression() {
    let mut assembler = Assembler::new();
    let operands = vec!["5ABCH SHR 8".to_string()];
    let data = assembler.handle_data_statement("DB", &Some(operands)).unwrap();
    assert_eq!(data, vec![0x5A]);
}

#[test]
fn db_test_string() {
    let mut assembler = Assembler::new();
    let operands = vec!["'STRINGSpl'".to_string()];
    let data = assembler.handle_data_statement("DB", &Some(operands)).unwrap();
    assert_eq!(data, vec![0x53, 0x54, 0x52, 0x49, 0x4E, 0x47, 0x53, 0x70, 0x6c]);
}

#[test]
fn db_test_minus() {
    let mut assembler = Assembler::new();
    let operands = vec!["-03H".to_string()];
    let data = assembler.handle_data_statement("DB", &Some(operands)).unwrap();
    assert_eq!(data, vec![0xFD]);
}

#[test]
fn db_test_multiple_operands_with_string() {
    let mut assembler = Assembler::new();
    let operands = vec!["13".to_string(),"12".to_string(),"'STRINGSpl'".to_string()];
    let data = assembler.handle_data_statement("DB", &Some(operands)).unwrap();
    assert_eq!(data, vec![13, 12, 0x53, 0x54, 0x52, 0x49, 0x4E, 0x47, 0x53, 0x70, 0x6c]);
}

#[test]
fn dw_test_single_value() {
    let mut assembler = Assembler::new();
    let operands = vec!["1234H".to_string()];
    let data = assembler.handle_data_statement("DW", &Some(operands)).unwrap();
    assert_eq!(data, vec![0x34, 0x12]);
}

#[test]
fn dw_test_multiple_operands() {
    let mut assembler = Assembler::new();
    let operands = vec!["1".to_string(), "2".to_string(), "3".to_string()];
    let data = assembler.handle_data_statement("DW", &Some(operands)).unwrap();
    assert_eq!(
        data,
        vec![0x01,0x00, 0x02,0x00, 0x03,0x00]
    );
}

#[test]
fn dw_test_expression() {
    let mut assembler = Assembler::new();
    let operands = vec!["100H + 2 * 10H".to_string()];
    let data = assembler.handle_data_statement("DW", &Some(operands)).unwrap();
    // 0x100 + 0x20 = 0x120
    assert_eq!(data, vec![0x20, 0x01]);
}

#[test]
fn dw_test_bitwise_expression() {
    let mut assembler = Assembler::new();
    let operands = vec!["0FF00H OR 0AAH".to_string()];
    let data = assembler.handle_data_statement("DW", &Some(operands)).unwrap();
    // 0xFF00 | 0x00AA = 0xFFAA
    assert_eq!(data, vec![0xAA, 0xFF]);
}

#[test]
fn dw_test_negative_value() {
    let mut assembler = Assembler::new();
    let operands = vec!["-1".to_string()];
    let data = assembler.handle_data_statement("DW", &Some(operands)).unwrap();
    assert_eq!(data, vec![0xFF, 0xFF]);
}

#[test]
fn dw_test_multiple_expressions_with_offset() {
    let mut assembler = Assembler::new();
    let operands = vec!["100H".to_string(), "HERE + 2".to_string()];
    let data = assembler.handle_data_statement("DW", &Some(operands)).unwrap();
    // HERE = 0
    // second operand offset = 2 → HERE + 2 = 2
    assert_eq!(data, vec![0x00,0x01, 0x02,0x00]);
}

#[test]
fn ds_test_simple() {
    let mut assembler = Assembler::new();
    let current_address = assembler.memory_pointer;
    let operands = vec!["10".to_string()];
    let data = assembler.handle_data_statement("DS", &Some(operands)).unwrap();
    assert_eq!(assembler.memory_pointer, current_address + 10);
}

#[test]
fn ds_test_expression() {
    let mut assembler = Assembler::new();
    let current_address = assembler.memory_pointer;
    let operands = vec!["5 * 4".to_string()];
    let data = assembler.handle_data_statement("DS", &Some(operands)).unwrap();
    assert_eq!(assembler.memory_pointer, current_address + (5 * 4));
}

#[test]
fn ds_test_negative_size_error() {
    let mut assembler = Assembler::new();
    let operands = vec!["-1".to_string()];
    assert!(assembler.handle_data_statement("DS", &Some(operands)).is_err());
}

#[test]
fn ds_test_here_expression() {
    let mut assembler = Assembler::new();
    let current_address = assembler.memory_pointer;
    let operands = vec!["HERE + 3".to_string()];
    let data = assembler.handle_data_statement("DS", &Some(operands)).unwrap();
    assert_eq!(assembler.memory_pointer, current_address + (current_address + 3));
}

#[test]
fn test_here_and_dollar_sign(){
    let mut assembler = Assembler::new();
    assert_eq!(assembler.calculate_expression("HERE",0, true).unwrap(), Some(0));
    assert_eq!(assembler.calculate_expression("HERE + 3",0, true).unwrap(), Some(3));
    assert_eq!(assembler.calculate_expression("$",0, true).unwrap(), Some(0));
    assert_eq!(assembler.calculate_expression("$ + 1100B",0, true).unwrap(), Some(12));
}

#[test]
fn assemble_file(){
    use std::fs;
    let file_path = "src/assembler/test_files/asm_test.asm";
    let file_content = fs::read_to_string(file_path).unwrap();
    let mut assembler = Assembler::new();
    let assembled_code = assembler.assemble(file_content.as_str()).unwrap();
}