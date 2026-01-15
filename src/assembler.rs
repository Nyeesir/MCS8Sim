#[cfg(test)]
mod assembler_tests;

use std::{error::Error, fmt, collections::HashMap};
use std::iter::Peekable;
use std::slice::Iter;
use regex::Regex;
//TODO: Dodac instrukcje rezerwacji przestrzeni, macro
//TODO: Dodac ewaluacje wyrazen arytmetycznych i logicznych jako operandow (strona 10) i dostosowac do tego parsowanie tokenow  -- chyba działa??
//TODO: Dodac zmienne przechowujace start i koniec programu -- chwilowo nie potrzebne ale pewnie przyda się w przyszłości
//TODO: W PRZYPADKU REJESTROW WALIDACJA CZY OPERAND WIEKSZY 0


const MEMORY_SIZE: usize = u16::MAX as usize + 1;

const INSTRUCTIONS: [&str; 78] = ["STC", "CMC", "INR", "DCR", "CMA", "DAA", "NOP", "MOV", "STAX", "LDAX"
    , "ADD", "ADC", "SUB", "SBB", "ANA", "XRA", "ORA", "CMP", "RLC", "RRC", "RAL", "RAR", "PUSH"
    , "POP", "DAD", "INX", "DCX", "XCHG", "XTHL", "SPHL", "LXI", "MVI", "ADI", "ACI", "SUI", "SBI", "ANI"
    , "XRI", "ORI", "CPI", "STA", "LDA", "SHLD", "LHLD", "PCHL", "JMP", "JC", "JNC", "JZ", "JNZ", "JP", "JM", "JPE", "JPO"
    , "CALL", "CC", "CNC", "CZ", "CNZ", "CP", "CM", "CPE", "CPO", "RET", "RC", "RNC", "RZ", "RNZ", "RM", "RP", "RPE", "RPO"
    , "RST", "EI", "DI", "IN", "OUT", "HLT"];
const PSEUDO_INSTRUCTIONS: [&str; 8] = ["ORG", "EQU", "SET", "END", "IF", "END IF", "MACRO", "END M"];
const DATA_STATEMENTS: [&str; 3] = ["DB", "DW", "DS"];

#[derive(Clone, Debug)]
#[derive(PartialEq)]
enum TokenType{
    Instruction,
    Operand,
    Label,
}
#[derive(Debug, Clone)]
pub struct AssemblyError{
    line_number: usize,
    line_text: String,
    message: String
}

impl Error for AssemblyError {}
impl fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error in line {} - {}:\n{}", self.line_number, self.line_text, self.message)
    }
}

#[derive(Debug, Clone)]
pub struct InvalidTokenError {
    token: String,
    token_type: TokenType,
    additional_info: Option<String>
}

impl Error for InvalidTokenError {}
impl fmt::Display for InvalidTokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut error_message: String;
        match self.token_type {
            TokenType::Instruction => error_message = "Invalid instruction".into(),
            TokenType::Operand => error_message = "Invalid operand".into(),
            TokenType::Label => error_message = "Invalid label".into(),
        }
        error_message.push_str(&format!(": {}.", self.token));
        if let Some(x) = &self.additional_info {
            error_message.push_str(&format!("Additional info: {}", x));
        }
        write!(f, "{}", error_message)
    }
}

#[derive(Debug, Clone)]
struct OverflowError;

impl Error for OverflowError {}
impl fmt::Display for OverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Memory overflow")
    }
}

pub struct Assembler{
    memory: [u8; MEMORY_SIZE],
    memory_pointer: usize,
    jump_map: HashMap<String, usize>,
    missing_jumps: HashMap<usize, String>,
    pending_exprs: Vec<PendingExpr>
}

#[derive(Debug, Clone)]
enum CalculationToken {
    Num(i32),
    Op(String),
    LParen,
    RParen,
    Label(String),
}

#[derive(Debug, Clone)]
enum Expr {
    Value(i32),
    Label(String),
    Unary { op: String, expr: Box<Expr> },
    Binary { op: String, left: Box<Expr>, right: Box<Expr> },
}

struct PendingExpr {
    addr: usize,
    expr: Expr,
}


impl Assembler{
    pub fn new() -> Self{
        Assembler{
            memory: [0; MEMORY_SIZE],
            memory_pointer: 0,
            jump_map: HashMap::new(),
            missing_jumps: HashMap::new(),
            pending_exprs: Vec::new()
        }
    }

    fn fetch_fields(line: &str) -> (Option<String>, Option<String>, Option<Vec<String>>){
        //RET label, instruction, operands; label and instruction are in upper case
        let mut ret = (None, None, None);
        let mut operands : Vec<String> = Vec::new();

        let line = line.split_whitespace().collect::<Vec<&str>>().join(" ");

        let mut word: String = String::new();
        let mut char_iter = line.chars().into_iter();

        //parsing first field (label or instruction)
        while let Some(char) = char_iter.next() {
            if char.is_whitespace(){
                break;
            } else {
                word.push(char);
            }
        }

        //check if the first token is a label or an instruction
        if !word.is_empty() {
            if word.ends_with(":"){
                //word is a label
                ret.0 = Some(word.to_uppercase());
            } else {
                //word is an instruction
                ret.1 = Some(word.to_uppercase());
            }
            word.clear();
        } else {
            return ret
        }


        //if instruction is not present, parse the second field assuming its instruction
        if ret.1.is_none() {
            while let Some(char) = char_iter.next() {
                if char.is_whitespace(){
                    break;
                } else {
                    word.push(char);
                }
            }
            if !word.is_empty() {
                ret.1 = Some(word.to_uppercase());
                word.clear();
            } else {
                return ret
            }
        }

        //adding operands to vector
        while let Some(char) = char_iter.next() {
            if char == ';'{
                break;
            }
            if char == ','{
                operands.push(word.trim().to_owned());
                word.clear();
            } else {
                word.push(char);
            }
        };
        if !word.is_empty() {operands.push(word.trim().to_owned())}

        ret.2 = Some(operands);
        return ret;
    }

    fn handle_fields(&mut self, label: &Option<String>, instruction: &Option<String>, operands: &Option<Vec<String>>) -> Result<(), AssemblyError>{
        if let Some(label) = label {
           Self::add_jump_point(self, &label[0..label.len()-1]).map_err(|e| AssemblyError { line_number: 0, line_text: "".into(), message: e.to_string() })?
        }

        if let Some(instruction) = instruction {
            match self.handle_data_statement(instruction, operands){
                Ok(values) => {
                    self.save_values_to_memory(values).map_err(|e| AssemblyError { line_number: 0, line_text: "".into(), message: e.to_string() })?;
                    return Ok(())
                },
                Err(e) if e.token_type == TokenType::Instruction => {},
                Err(e) => return Err(AssemblyError { line_number: 0, line_text: "".into(), message: e.to_string() })
            }

            match self.translate_instruction(instruction, operands){
                Ok(values) => {
                    self.save_values_to_memory(values).map_err(|e| AssemblyError { line_number: 0, line_text: "".into(), message: e.to_string() })?;
                    return Ok(())
                },
                Err(e) => return Err(AssemblyError { line_number: 0, line_text: "".into(), message: e.to_string() })
            }
        }
        Ok(())

    }

    pub fn assemble (&mut self, data: &str) -> Result<[u8; MEMORY_SIZE], AssemblyError> {
        let mut line_number: usize = 0;

        let lines = data.lines();
        for line in lines{
            line_number += 1;
            let line = line.trim();
            if line.is_empty() {continue}

            let (label, instruction, operands) = Self::fetch_fields(&line);

            self.handle_fields(&label, &instruction, &operands)?;
        }

        self.resolve_pending_exprs()
            .map_err(|e| AssemblyError {
                line_number,
                line_text: "FIXME".into(),
                message: e.to_string(),
            })?;
        Ok(self.memory)
    }

    fn save_values_to_memory(&mut self, values: Vec<u8>) -> Result<(), OverflowError>{
        for value in values{
            self.memory[self.memory_pointer] = value;
            self.memory_pointer += 1;
            if self.memory_pointer >= MEMORY_SIZE {
                return Err(OverflowError)
            }
        }
        Ok(())
    }

    fn translate_instruction(&mut self, instruction: &str, operands: &Option<Vec<String>>) -> Result<Vec<u8>, InvalidTokenError>{
        let instruction_in_upper = instruction.to_uppercase();
        let instruction = instruction_in_upper.as_str();

        let mut binary_values: Vec<u8> = Vec::with_capacity(3);
        match instruction {
            "STC" => binary_values.push(0b00110111),
            "CMC" => binary_values.push(0b00111111),
            "INR" => {
                binary_values.push(0b00000100);
                let operands = Self::assert_operand_amount(operands, 1)?;
                let register = Self::parse_register(operands[0].as_str())?;
                binary_values[0] |= register << 3;
            }
            "DCR" => {
                binary_values.push(0b00000101);
                let operands = Self::assert_operand_amount(operands, 1)?;
                let register = Self::parse_register(operands[0].as_str())?;
                binary_values[0] |= register << 3;
            }
            "CMA" => binary_values.push(0b00101111),
            "DAA" => binary_values.push(0b00100111),
            "NOP" => binary_values.push(0b00000000),
            "MOV" => {
                binary_values.push(0b01000000);
                let operands = Self::assert_operand_amount(operands, 2)?;
                let (left_register, right_register) = (Self::parse_register(operands[0].as_str())?, Self::parse_register(operands[1].as_str())?);
                binary_values[0] |= (left_register << 3) | right_register;
            }
            "STAX" | "LDAX" => {
                let operands = Self::assert_operand_amount(operands, 1)?;
                let register_pair = Self::parse_register_pair(operands[0].as_str())?;
                match operands[0].as_str() {
                    "BC" | "B" | "DE" | "D" => {}
                    _ => return Err(InvalidTokenError { token: operands[0].clone(), token_type: TokenType::Operand, additional_info: Some("Only BC, B, DE, D are allowed".into())})
                }
                match instruction {
                    "STAX" => binary_values.push(0b00000010),
                    "LDAX" => binary_values.push(0b00001010),
                    _ => unreachable!()
                }
                binary_values[0] |= register_pair<<4;
            }
            "ADD" | "ADC" | "SUB" | "SBB" | "ANA" | "XRA" | "ORA" | "CMP" => {
                binary_values.push(0b10000000);
                match instruction {
                    "ADD" => binary_values[0] |= 0b000000,
                    "ADC" => binary_values[0] |= 0b001000,
                    "SUB" => binary_values[0] |= 0b010000,
                    "SBB" => binary_values[0] |= 0b011000,
                    "ANA" => binary_values[0] |= 0b100000,
                    "XRA" => binary_values[0] |= 0b101000,
                    "ORA" => binary_values[0] |= 0b110000,
                    "CMP" => binary_values[0] |= 0b111000,
                    _ => unreachable!()
                }
                let operands = Self::assert_operand_amount(operands, 1)?;
                let register = Self::parse_register(operands[0].as_str())?;
                binary_values[0] |= register;
            }
            "RLC" => binary_values.push(0b00000111),
            "RRC" => binary_values.push(0b00001111),
            "RAL" => binary_values.push(0b00010111),
            "RAR" => binary_values.push(0b00011111),
            "PUSH" => {
                binary_values.push(0b11000101);
                let operands = Self::assert_operand_amount(operands, 1)?;
                let register_pair = Self::parse_register_pair(operands[0].as_str())?;
                binary_values[0] |= register_pair<<4;
            }
            "POP" => {
                binary_values.push(0b11000001);
                let operands = Self::assert_operand_amount(operands, 1)?;
                let register_pair = Self::parse_register_pair(operands[0].as_str())?;
                match operands[0].as_str() {
                    "Bc" | "B" | "DE" | "D" | "H" | "HL" | "PSW" => {}
                    _ => return Err(InvalidTokenError { token: operands[0].clone(), token_type: TokenType::Operand, additional_info: Some("Only BC, B, DE, D are allowed".into())})
                }
                binary_values[0] |= register_pair<<4;
            }
            "DAD" => {
                binary_values.push(0b00001001);
                let operands = Self::assert_operand_amount(operands, 1)?;
                let register_pair = Self::parse_register_pair(operands[0].as_str())?;
                binary_values[0] |= register_pair<<4;
            }
            "INX" => {
                binary_values.push(0b00000011);
                let operands = Self::assert_operand_amount(operands, 1)?;
                let register_pair = Self::parse_register_pair(operands[0].as_str())?;
                binary_values[0] |= register_pair<<4;
            }
            "DCX" => {
                binary_values.push(0b00001011);
                let operands = Self::assert_operand_amount(operands, 1)?;
                let register_pair = Self::parse_register_pair(operands[0].as_str())?;
                binary_values[0] |= register_pair<<4;
            }
            "XCHG" => binary_values.push(0b11101011),
            "XTHL" => binary_values.push(0b11100011),
            "SPHL" => binary_values.push(0b11111001),
            "LXI" => {
                binary_values.push(0b00000001);
                let operands = Self::assert_operand_amount(operands, 2)?;
                let (register_pair, operand) = (operands[0].as_str(), operands[1].as_str());
                let register_pair = Self::parse_register_pair(register_pair)?;
                binary_values[0] |= register_pair << 4;
                let addr = self.parse_16bit_expr(operand,1)?;
                binary_values.push(addr.0);
                binary_values.push(addr.1);
            }
            "MVI" => {
                binary_values.push(0b00000110);
                let operands = Self::assert_operand_amount(operands, 2)?;
                let (register, operand) = (operands[0].as_str(), operands[1].as_str());
                let register = Self::parse_register(register)?;
                binary_values[0] |= register << 3;
                binary_values.push(self.parse_8bit_expr(&operand,1)?);
            }
            "ADI" | "ACI" | "SUI" | "SBI" | "ANI" | "XRI" | "ORI" | "CPI" => {
                binary_values.push(0b11000110);
                match instruction {
                    "ADI" => binary_values[0] |= 0b000110,
                    "ACI" => binary_values[0] |= 0b001110,
                    "SUI" => binary_values[0] |= 0b010110,
                    "SBI" => binary_values[0] |= 0b011110,
                    "ANI" => binary_values[0] |= 0b100110,
                    "XRI" => binary_values[0] |= 0b101110,
                    "ORI" => binary_values[0] |= 0b110110,
                    "CPI" => binary_values[0] |= 0b111110,
                    _ => unreachable!()
                }
                let operands = Self::assert_operand_amount(operands, 1)?;
                binary_values.push(self.parse_8bit_expr(operands[0].as_str(),1)?);
                //TODO: BRAKUJE PARSOWANIA WARTOSCI???
                //JAKIS STARY KOMENTARZ KTOREGO NIE ROZUMIEM, NA RAZIE ZOSTAWIE
            }
            "STA" | "LDA" | "SHLD" | "LHLD" => {
                binary_values.push(0b00100010);
                match instruction {
                    "STA" => binary_values[0] |= 0b10010,
                    "LDA" => binary_values[0] |= 0b11010,
                    "SHLD" => binary_values[0] |= 0b00010,
                    "LHLD" => binary_values[0] |= 0b01010,
                    _ => unreachable!()
                }
                let operands = Self::assert_operand_amount(operands, 1)?;
                let addr = self.parse_16bit_expr(operands[0].as_str(),1)?;
                binary_values.push(addr.0);
                binary_values.push(addr.1);
            }
            "PCHL" => binary_values.push(0b11101001),
            "JMP" | "JNZ" | "JZ" | "JNC" | "JC" | "JM" | "JP" | "JPE" | "JPO" => {
                binary_values.push(0b11000010);
                match instruction {
                    "JMP" => binary_values[0] |= 0b000011,
                    "JNZ" => binary_values[0] |= 0b000010,
                    "JZ" => binary_values[0] |= 0b001010,
                    "JNC" => binary_values[0] |= 0b010010,
                    "JC" => binary_values[0] |= 0b011010,
                    "JPO" => binary_values[0] |= 0b100010,
                    "JPE" => binary_values[0] |= 0b101010,
                    "JP" => binary_values[0] |= 0b110010,
                    "JM" => binary_values[0] |= 0b111010,
                    _ => unreachable!()
                }
                let operands = Self::assert_operand_amount(operands, 1)?;
                let addr = self.parse_16bit_expr(operands[0].as_str(),1)?;
                binary_values.push(addr.0);
                binary_values.push(addr.1);
            }
            "CNZ" | "CZ" | "CALL" | "CNC" | "CC" | "CPO" | "CPE" | "CP" | "CM" => {
                binary_values.push(0b11000100);
                match instruction {
                    "CNZ" => binary_values[0] |= 0b000100,
                    "CZ" => binary_values[0] |= 0b001100,
                    "CALL" => binary_values[0] |= 0b001101,
                    "CNC" => binary_values[0] |= 0b010100,
                    "CC" => binary_values[0] |= 0b011100,
                    "CPO" => binary_values[0] |= 0b100100,
                    "CPE" => binary_values[0] |= 0b101100,
                    "CP" => binary_values[0] |= 0b110100,
                    "CM" => binary_values[0] |= 0b111100,
                    _ => unreachable!()
                }
                let operands = Self::assert_operand_amount(operands, 1)?;
                let addr = self.parse_16bit_expr(operands[0].as_str(),1)?;
                binary_values.push(addr.0);
                binary_values.push(addr.1);
            }
            "RET" => binary_values.push(0b11001001),
            "RC" => binary_values.push(0b11011000),
            "RNC" => binary_values.push(0b11010000),
            "RZ" => binary_values.push(0b11001000),
            "RNZ" => binary_values.push(0b11000000),
            "RM" => binary_values.push(0b11111000),
            "RP" => binary_values.push(0b11110000),
            "RPE" => binary_values.push(0b11101000),
            "RPO" => binary_values.push(0b11100000),
            "RST" => {
                binary_values.push(0b11000111);
                let operands = Self::assert_operand_amount(operands, 1)?;
                match Self::parse_8bit_number(operands[0].as_str()) {
                    Ok(x) => {
                        if x < 8 {
                            binary_values[0] |= x<<3;
                        } else {
                            return Err(InvalidTokenError { token: operands[0].clone(), token_type: TokenType::Operand, additional_info: Some("RST number is out of range".into())})
                        }
                    }
                    Err(_) => return Err(InvalidTokenError { token: operands[0].clone(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within u8 range are allowed".into())})
                }
            }
            "EI" => binary_values.push(0b11111011),
            "DI" => binary_values.push(0b11110011),
            "IN" | "OUT" => {
                binary_values.push(0b11010011);
                match instruction {
                    "IN" => binary_values[0] |= 0b0000100,
                    "OUT" => binary_values[0] |= 0b0000000,
                    _ => unreachable!()
                }
                let operands = Self::assert_operand_amount(&operands, 1)?;
                match self.parse_8bit_expr(operands[0].as_str(),1) {
                    Ok(x) => binary_values.push(x),
                    Err(_) => return Err(InvalidTokenError { token: operands[0].clone(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within u8 range are allowed".into())})
                }
            }
            "HLT" => binary_values.push(0b01110110),
            _ => return Err(InvalidTokenError { token: instruction.into(), token_type: TokenType::Instruction, additional_info: None})
        }
        Ok(binary_values)
    }

    fn parse_register(operand: &str) -> Result<u8, InvalidTokenError>{
        let register_in_upper = operand.to_uppercase();
        let register = register_in_upper.as_str();
        match register {
            "B" => Ok(0b000),
            "C" => Ok(0b001),
            "D" => Ok(0b010),
            "E" => Ok(0b011),
            "H" => Ok(0b100),
            "L" => Ok(0b101),
            "M" => Ok(0b110),
            "A" => Ok(0b111),
            _ => {
                match Self::parse_8bit_number(register){
                    Ok(x) => {
                        if x < 8 {
                            Ok(x)
                        } else { Err(InvalidTokenError { token: register.into(), token_type: TokenType::Operand, additional_info: Some("Register number is out of range".into())}) }
                    }
                    Err(_) => Err(InvalidTokenError { token: register.into(), token_type: TokenType::Operand, additional_info: Some("Only registers as words or their numeric presentation is allowed".into())})
                }
            }
        }
    }

    fn parse_register_pair(operand: &str) -> Result<u8, InvalidTokenError>{
        let register_pair_in_upper = operand.to_uppercase();
        let register_pair = register_pair_in_upper.as_str();
        match register_pair {
            "BC" | "B" => Ok(0b00),
            "DE" | "D" => Ok(0b01),
            "HL" | "H" => Ok(0b10),
            "SP" | "PSW" => Ok(0b11),
            _ => {
                Err(InvalidTokenError { token: register_pair.into(), token_type: TokenType::Operand, additional_info: Some("Only register pairs as words are allowed".into())})
            }
        }
    }

    fn parse_16bit_expr(&mut self, expr: &str, offset: usize) -> Result<(u8, u8), InvalidTokenError> {
        match self.calculate_expression(expr, offset)? {
            Some(v) => {
                if (-32768..=65535).contains(&v) {
                    let val = v as i16 as u16;
                    Ok(val.to_le_bytes().into())
                } else {
                    Err(InvalidTokenError {
                        token: expr.into(),
                        token_type: TokenType::Operand,
                        additional_info: Some("Expression does not fit in signed 16 bits".into()),
                    })
                }
            }
            None => Ok((0, 0)),
        }
    }

    //FIXME: liczby ujemne ze swojej natury wypelniaja caly zakres, przez co wychodza poza zakres u8 i mamy problem
    fn parse_8bit_expr(&mut self, expr: &str, offset: usize) -> Result<u8, InvalidTokenError> {
        match self.calculate_expression(expr, offset)? {
            Some(v) => {
                if (-128..=255).contains(&v) {
                    Ok(v as i8 as u8)
                } else {
                    Err(InvalidTokenError {
                        token: expr.into(),
                        token_type: TokenType::Operand,
                        additional_info: Some("Expression does not fit in signed 8 bits".into()),
                    })
                }
            }
            None => Ok(0),
        }
    }

    fn parse_number_i32(number: &str) -> Result<i32, InvalidTokenError>{
        let value = number.to_uppercase();

        let (number, radix) = match value.chars().last() {
            Some('O') | Some('Q') => (&value[0..value.len()-1], 8),
            Some('B') => (&value[0..value.len()-1], 2),
            Some('H') if value.starts_with(&['-','0','1','2','3','4','5','6','7','8','9']) => (&value[0..value.len()-1], 16),
            Some('D') => (&value[0..value.len()-1], 10),
            Some(_) => (value.as_str(), 10),
            None => {Err(InvalidTokenError { token: value.clone(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within valid range with right suffixes are allowed".into())})}?
        };

        i32::from_str_radix(number, radix).map_err(|_| InvalidTokenError { token: value.into(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within valid range with right suffixes are allowed".into())})
    }

    fn parse_8bit_number(number: &str) -> Result<u8, InvalidTokenError>{
        match Self::parse_number_i32(number){
            Ok(x) => {
                let v = x as i16;
                if (i8::MIN as i16..= u8::MAX as i16).contains(&v) {
                    Ok(v as u8)
                } else {
                    Err(InvalidTokenError { token: number.into(), token_type: TokenType::Operand, additional_info: Some("Only 8-bit numeric values with right suffixes are allowed".into())})
                }
            }
            Err(_) => Err(InvalidTokenError { token: number.into(), token_type: TokenType::Operand, additional_info: Some("Only 8-bit numeric values with right suffixes are allowed".into())})
        }
    }

    fn add_jump_point(&mut self, label: &str) -> Result<(), InvalidTokenError> {
        match self.validate_label(label) {
            Ok(()) => {},
            Err(e) => return Err(e)
        }

        if self.jump_map.contains_key(label){
            return Err(InvalidTokenError {token: label.into(), token_type: TokenType::Label, additional_info: Some("Label already exists".into())})
        }

        self.jump_map.insert(label.into(), self.memory_pointer);
        Ok(())
    }

    fn validate_label(&self, label: &str) -> Result<(), InvalidTokenError>{
        //We should allow labels with max 5 chars, but we will skip it for now
        if !label.is_ascii() {return Err(InvalidTokenError { token: label.into(), token_type: TokenType::Label, additional_info: Some("Labels can only contain ASCII characters".into())})}

        let first_char = label.chars().next().ok_or(InvalidTokenError { token: label.into(), token_type: TokenType::Label, additional_info: Some("Label is empty".into())})?;
        if !['@', '?', ':'].contains(&first_char) && !first_char.is_ascii_alphabetic() {return Err(InvalidTokenError { token: label.into(), token_type: TokenType::Label, additional_info: Some("Labels cannot begin with a decimal digit or special character".into())});}

        if INSTRUCTIONS.contains(&label) || PSEUDO_INSTRUCTIONS.contains(&label){ return Err(InvalidTokenError { token: label.into(), token_type: TokenType::Label, additional_info: Some("Labels cannot be the same as an instruction or a pseudo-instruction".into())});}

        Ok(())
    }

    fn assert_operand_amount(operands: &Option<Vec<String>>, allowed_amount: usize) -> Result<&Vec<String>, InvalidTokenError>{
        match operands {
            Some(operands) => {
                if operands.len() < allowed_amount{
                    return Err(InvalidTokenError { token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Too few operands".into())})
                } else if operands.len() > allowed_amount{
                    return Err(InvalidTokenError { token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Too many operands".into())})
                } else {
                    return Ok(operands)
                }
            }
            None => return Err(InvalidTokenError { token: "".into(), token_type: TokenType::Operand, additional_info: Some("Too few operands".into())})
        }
    }

    fn handle_pseudo_instruction(&mut self, label: &str, instruction: &str, operands: &Vec<&str>) -> Result<(), InvalidTokenError>{
        match instruction {
            "COSTAM" => unimplemented!(),
            _ => Err( InvalidTokenError {token: instruction.into(), token_type:TokenType::Instruction, additional_info: Some("It is not a valid pseudo-instruction".into())})
        }
    }

    fn handle_macro() -> Result<(), InvalidTokenError>{
        unimplemented!()
    }

    fn handle_data_statement(&mut self, instruction: &str, operands: &Option<Vec<String>>) -> Result<Vec<u8>, InvalidTokenError>{
        let mut values = Vec::new();
        match instruction {
            "DB" => {
                let operands = if let Some(operands) = operands {
                    operands
                } else {
                    Err(InvalidTokenError {
                        token: instruction.into(),
                        token_type: TokenType::Operand,
                        additional_info: Some("Missing operands".into()),
                    })?
                };


                let mut offset = 0;
                for operand in operands{
                    if operand.len() > 3  && operand.starts_with("'") && operand.ends_with("'"){
                        for char in operand.strip_prefix("'").unwrap().strip_suffix("'").unwrap().chars(){
                            if char.is_ascii(){
                                values.push(char as u8);
                                offset += 1;
                            } else {
                                return Err(InvalidTokenError {token: operand.into(), token_type: TokenType::Operand, additional_info: Some("String contains non-ASCII characters".into())})
                            }
                        }
                    } else {
                        values.push(self.parse_8bit_expr(operand, offset)?);
                        offset += 1;
                    }
                }
                Ok(values)

            },
            "DW" => unimplemented!(),
            "DS" => unimplemented!(),
            _ => Err( InvalidTokenError {token: instruction.into(), token_type:TokenType::Instruction, additional_info: Some("It is not a valid data statement".into())})
        }
    }

    /*TODO:
    HERE AND $ - CURRENT ADDRESS -- should work
    i16? u16? u8? how am i supposed to do it -- should be fine for now
    how to handle ascii and labels -- TODO: most likely wrapper that check if first

    for example u8 might use expressions
    MVI, H,NOT 0 is not valid because its 16bit 0FFFFH
    MVI, H,NOT 0 AND OFFH i valid because its 8bit 0FFH

    INS: DB (ADD C) should be theoretically valid, how to handle it -- TODO: most likely skip

    All operators treat their arguments as 15-bit quantities, and generate 16-bit quantities as their result ???????
    what does it even mean

    what values should parser take, is parsing i16 correct? i dont think so
    FIXME: how to handle labels in equations when they're not in the jump map????????
    */


    //FIXME: OGARNAC PRAWIDLOWY RANGE DLA TEGO SYFU CALEGO
    fn calculate_expression(
        &mut self,
        expr: &str,
        offset: usize,
    ) -> Result<Option<i32>, InvalidTokenError> {
        let tokens = Self::tokenize(self, expr)?;
        let mut it = tokens.iter().peekable();

        let ast = Self::parse_expr(&mut it, 0)?;

        if it.peek().is_some() {
            return Err(InvalidTokenError {
                token: expr.into(),
                token_type: TokenType::Operand,
                additional_info: Some("Unexpected token at end of expression".into()),
            });
        }

        match self.eval_expr(&ast) {
            Ok(v) => Ok(Some(v)),
            Err(_) => {
                self.pending_exprs.push(PendingExpr {
                    addr: self.memory_pointer+offset,
                    expr: ast,
                });
                Ok(None)
            }
        }
    }

    fn tokenize(&self, expr: &str) -> Result<Vec<CalculationToken>, InvalidTokenError> {
        let pattern = r"(\bHERE\b|\$|\bMOD\b|\bNOT\b|\bAND\b|\bOR\b|\bXOR\b|\bSHL\b|\bSHR\b|\+|\-|\*|/|\(|\))";
        let re = Regex::new(pattern).unwrap();

        let mut tokens = Vec::new();
        let mut last = 0;

        for m in re.find_iter(expr) {
            if m.start() > last {
                Self::push_part_as_token(&expr[last..m.start()], &mut tokens);
            }

            let t = m.as_str();
            tokens.push(match t {
                "(" => CalculationToken::LParen,
                ")" => CalculationToken::RParen,
                "HERE" | "$" => CalculationToken::Num(self.memory_pointer as i32),
                _ => CalculationToken::Op(t.to_string()),
            });

            last = m.end();
        }

        if last < expr.len() {
            Self::push_part_as_token(&expr[last..], &mut tokens);
        }

        Ok(tokens)
    }

    fn push_part_as_token(
        part: &str,
        tokens: &mut Vec<CalculationToken>,
    ) {
        let part = part.trim();
        if part.is_empty() {
            return;
        }

        if part.starts_with('\'') && part.ends_with('\'') && part.len() == 2 {
            tokens.push(CalculationToken::Num(0i32));
        }
        else if part.starts_with('\'') && part.ends_with('\'') && part.len() == 3 {
            let c = part.chars().nth(1).unwrap();
            tokens.push(CalculationToken::Num(c as i32));
        } else if let Ok(v) = Self::parse_number_i32(part) {
            tokens.push(CalculationToken::Num(v as i32));
        } else {
            tokens.push(CalculationToken::Label(part.to_string()));
        }
    }

    fn precedence(op: &str) -> u8 {
        match op {
            "OR" | "XOR" => 1,
            "AND"        => 2,
            "NOT"        => 3, // unarny
            "+" | "-"    => 4, // binarny
            "*" | "/" | "MOD" | "SHL" | "SHR" => 5,
            _ => 0,
        }
    }

    fn parse_expr(
        tokens: &mut Peekable<Iter<CalculationToken>>,
        min_prec: u8,
    ) -> Result<Expr, InvalidTokenError> {

        let mut lhs = match tokens.next() {
            Some(CalculationToken::Num(v)) => Expr::Value(*v),

            Some(CalculationToken::Label(l)) => Expr::Label(l.clone()),

            Some(CalculationToken::Op(op)) if op == "-" || op == "NOT" => {
                let prec = Self::precedence(op);
                let expr = Self::parse_expr(tokens, prec)?;
                Expr::Unary {
                    op: op.clone(),
                    expr: Box::new(expr),
                }
            }

            Some(CalculationToken::LParen) => {
                let e = Self::parse_expr(tokens, 0)?;
                match tokens.next() {
                    Some(CalculationToken::RParen) => e,
                    _ => return Err(InvalidTokenError {
                        token: "".into(),
                        token_type: TokenType::Operand,
                        additional_info: Some("Missing ')'".into()),
                    }),
                }
            }

            _ => return Err(InvalidTokenError {
                token: "".into(),
                token_type: TokenType::Operand,
                additional_info: Some("Expected expression".into()),
            }),
        };

        while let Some(CalculationToken::Op(op)) = tokens.peek() {
            let prec = Self::precedence(op);
            if prec < min_prec {
                break;
            }

            let op = op.clone();
            tokens.next();

            let rhs = Self::parse_expr(tokens, prec + 1)?;
            lhs = Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn eval_bin(op: &str, a: i32, b: i32) -> i32 {
        let r = match op {
            "+" => a.wrapping_add(b),
            "-" => a.wrapping_sub(b),
            "*" => a.wrapping_mul(b),
            "/" => a / b,
            "MOD" => a % b,
            "AND" => a & b,
            "OR"  => a | b,
            "XOR" => a ^ b,
            "SHL" => a.wrapping_shl((b & 0xF) as u32),
            "SHR" => a.wrapping_shr((b & 0xF) as u32),
            _ => unreachable!(),
        };
        r & 0xFFFF
    }

    fn eval_expr(&self, expr: &Expr) -> Result<i32, String> {
        match expr {
            Expr::Value(v) => Ok(*v),

            Expr::Label(l) => {
                self.jump_map
                .get(&l.to_uppercase())
                .map(|v| *v as i32)
                .ok_or(format!("Undefined label {}", l))},

            Expr::Unary { op, expr } => {
                let v = self.eval_expr(expr)?;
                Ok(match op.as_str() {
                    "NOT" => !v & 0xFFFF,
                    "-" => (!v).wrapping_add(1),
                    _ => unreachable!(),
                })
            }

            Expr::Binary { op, left, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                Ok(Self::eval_bin(op, l, r))
            }
        }
    }

    fn resolve_pending_exprs(&mut self) -> Result<(), InvalidTokenError> {
        for p in &self.pending_exprs {
            let v = self.eval_expr(&p.expr)
                .map_err(|e| InvalidTokenError {
                    token: e,
                    token_type: TokenType::Label,
                    additional_info: None,
                })?;

            let b = v.to_le_bytes();
            self.memory[p.addr] = b[0];
            self.memory[p.addr + 1] = b[1];
        }
        Ok(())
    }
}