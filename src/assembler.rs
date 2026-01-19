#[cfg(test)]
mod assembler_tests;
mod errors;

use std::{error::Error, fmt, collections::HashMap};
use std::iter::Peekable;
use std::slice::Iter;
use std::str::Chars;
use regex::Regex;
use errors::{AssemblyError, OverflowError, InvalidTokenError, TokenOrOverflowError, InvalidTokenAtLineError, TokenType};
/*
TODO:
INS: DB (ADD C) nie jest obecnie możliwe, chyba do olania
DS i ORG nie działają do końca jak powinny przy operandach, nie obsługują forward referencing
W przypadku rejestrow nie przyjmujemy wyrażeń a tylko stałe w postaci odpowiednich stringów lub cyfr w przypadku pojedynczych rejestrów
 */


//TODO: Obsłużyć resztę pseudo-instrukcji
//TODO: Dodac zmienne przechowujace start i koniec programu -- chwilowo nie potrzebne ale pewnie przyda się w przyszłości


const MEMORY_SIZE: usize = u16::MAX as usize + 1;

const INSTRUCTIONS: [&str; 78] = ["STC", "CMC", "INR", "DCR", "CMA", "DAA", "NOP", "MOV", "STAX", "LDAX"
    , "ADD", "ADC", "SUB", "SBB", "ANA", "XRA", "ORA", "CMP", "RLC", "RRC", "RAL", "RAR", "PUSH"
    , "POP", "DAD", "INX", "DCX", "XCHG", "XTHL", "SPHL", "LXI", "MVI", "ADI", "ACI", "SUI", "SBI", "ANI"
    , "XRI", "ORI", "CPI", "STA", "LDA", "SHLD", "LHLD", "PCHL", "JMP", "JC", "JNC", "JZ", "JNZ", "JP", "JM", "JPE", "JPO"
    , "CALL", "CC", "CNC", "CZ", "CNZ", "CP", "CM", "CPE", "CPO", "RET", "RC", "RNC", "RZ", "RNZ", "RM", "RP", "RPE", "RPO"
    , "RST", "EI", "DI", "IN", "OUT", "HLT"];
const PSEUDO_INSTRUCTIONS: [&str; 8] = ["ORG", "EQU", "SET", "END", "IF", "ENDIF", "MACRO", "ENDM"];
const DATA_STATEMENTS: [&str; 3] = ["DB", "DW", "DS"];

pub struct Assembler{
    memory: [u8; MEMORY_SIZE],
    memory_pointer: usize,
    symbols: HashMap<String, Symbol>,
    pending_exprs: Vec<PendingExpr>,
    stopped: bool,
    current_line: usize,
    if_stack: Vec<bool>,
}

enum SymbolKind {
    Label,
    Equ,
    Set,
}

struct Symbol {
    value: i32,
    kind: SymbolKind,
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
    line: usize
}


impl Assembler{
    pub fn new() -> Self{
        Assembler{
            memory: [0; MEMORY_SIZE],
            memory_pointer: 0,
            symbols: HashMap::new(),
            pending_exprs: Vec::new(),
            stopped: false,
            current_line: 0,
            if_stack: Vec::new(),
        }
    }

    fn advance_to_next_no_space_char(char_iter: &mut Peekable<Chars>){
        while let Some(c) = char_iter.peek() {
            if !c.is_ascii_whitespace() { break }
            char_iter.next();
        }
    }

    fn read_token_to_uppercase_to_nearest_space(char_iter: &mut Peekable<Chars>) -> String{
        let mut word = String::new();
        while let Some(c) = char_iter.next() {
            if c.is_ascii_whitespace() { break }
            word.push(c.to_ascii_uppercase());
        }
        word
    }


    fn fetch_fields(line: &str) -> (Option<String>, Option<String>, Option<Vec<String>>){
        //RET label, instruction, operands; label and instruction are in upper case
        let mut ret = (None, None, None);
        let mut operands : Vec<String> = Vec::new();

        let mut line = line.trim();

        //removes comments
        if let Some((fields, _)) = line.split_once(";"){line = fields}
        if line.is_empty() { return ret }


        let mut char_iter = line.chars().peekable();
        let mut field = String::new();

        //parse first word
        field = Self::read_token_to_uppercase_to_nearest_space( &mut char_iter);

        if field.is_empty() { return ret }

        Self::advance_to_next_no_space_char(&mut char_iter);

        //check if the first word is an instruction, if not, it is a label
        if INSTRUCTIONS.contains(&field.as_str()) || PSEUDO_INSTRUCTIONS.contains(&field.as_str()) || DATA_STATEMENTS.contains(&field.as_str()) {
            ret.1 = Some(field.clone());
        } else {
            ret.0 = Some(field.clone());
        }
        field.clear();

        //if the instruction value is none, then we parse the second word assuming it's an instruction
        if ret.1.is_none() {
            field = Self::read_token_to_uppercase_to_nearest_space( &mut char_iter);

            //if the instruction field is empty, then we return with only the label set
            if field.is_empty() {
                return ret
            }
            //else we set our word as ret.1
            else {
                ret.1 = Some(field.trim().to_string());
                field.clear();
            }
        }

        Self::advance_to_next_no_space_char(&mut char_iter);

        //handling operands
        let mut is_inside_string = false;
        while let Some(c) = char_iter.next() {
            match c {
                '\'' => {
                    field.push(c);
                    is_inside_string = !is_inside_string;
                }
                ',' if is_inside_string => {
                    field.push(c);
                }
                ',' if !is_inside_string => {
                    operands.push(field.trim().to_string());
                    field.clear();
                }
                _ if is_inside_string => {
                    field.push(c);
                }
                _ => {
                    field.push(c.to_ascii_uppercase());
                }
            }
        }

        //adding to an operand list remaining operand
        if !field.is_empty() {operands.push(field.trim().to_string())}
        ret.2 = Some(operands);

        println!("{:?}", ret);
        ret
    }

    fn is_compiling(&self) -> bool {
        self.if_stack.iter().all(|&v| v)
    }

    fn handle_fields(&mut self, label: &Option<String>, instruction: &Option<String>, operands: &Option<Vec<String>>) -> Result<(), TokenOrOverflowError>{
        if let Some(instruction) = instruction {
            if instruction == "IF" {
                self.handle_if_instruction(instruction, operands)?;
                return Ok(())
            }
            else if instruction == "ENDIF" {
                self.handle_endif_instruction(instruction, operands)?;
                return Ok(())
            }
        }

        if !self.is_compiling() {
            return Ok(())
        }

        if let Some(label) = label {
            if label.ends_with(":") {
               Self::define_label(self, &label[0..label.len()-1])?
            } else {
                match instruction.as_deref() {
                    Some("SET" | "EQU") => {}
                    _ => {
                        return Err(
                            InvalidTokenError {token: label.clone(), token_type: TokenType::Label, additional_info: Some("Only SET and EQU take labels without colons".into())}.into()
                        )
                    }
                }
            }
        }

        if let Some(instruction) = instruction {
            match self.handle_data_statement(instruction, operands){
                Ok(values) => {
                    return match values {
                        Some(values) => {
                            self.save_values_to_memory(values)?;
                            Ok(())
                        }
                        None => Ok(())
                    }
                },
                Err(e) => {
                    match e {
                        TokenOrOverflowError::InvalidToken(e) if e.token_type == TokenType::Instruction => {}
                        TokenOrOverflowError::InvalidToken(e) => return Err(e.into()),
                        TokenOrOverflowError::Overflow(e) => return Err(e.into())
                    }
                }
            }

            match self.handle_pseudo_instruction(label, instruction, operands){
                Ok(_) => { return Ok(()) },
                Err(e) if e.token_type == TokenType::Instruction => {},
                Err(e) => return Err(e.into())
            }

            return match self.handle_instruction(instruction, operands) {
                Ok(values) => {
                    Ok(self.save_values_to_memory(values)?)
                },
                Err(e) => Err(e.into())
            }
        }
        Ok(())

    }

    pub fn assemble (&mut self, data: &str) -> Result<[u8; MEMORY_SIZE], AssemblyError> {

        let lines = data.lines();
        for line in lines {
            self.current_line += 1;
            let line_number = self.current_line;
            let line = line.trim();
            if line.is_empty() { continue }
            if !line.is_ascii() { return Err(AssemblyError { line_number, line_text: line.into(), message: "Non-ASCII characters found".into() })}

            let (label, instruction, operands) = Self::fetch_fields(&line);

            match self.handle_fields(&label, &instruction, &operands) {
                Ok(_) => {}
                Err(TokenOrOverflowError::Overflow(_)) => {
                    return Err(AssemblyError { line_number, line_text: line.into(), message: "Overflow".into() })
                }
                Err(TokenOrOverflowError::InvalidToken(e)) => {
                    return Err(AssemblyError { line_number, line_text: line.into(), message: e.to_string() })
                }
            }

            if self.stopped { break }
        }

        if !self.if_stack.is_empty() {
            //TODO: ZASTANOWIC SIE CZY TAK ZOSTAWIC
            return Err(AssemblyError {
                line_number: 0,
                line_text: "".to_string(),
                message: "Unclosed IF block".into(),
            });
        }

        self.resolve_pending_exprs()
            .map_err(|e| AssemblyError {
                line_number: e.line,
                line_text: data.lines().nth(e.line).or_else(|| Some("")).unwrap_or_default().into(),
                message: e.source.to_string()
            })?;

        Ok(self.memory)
    }

    fn save_values_to_memory(&mut self, values: Vec<u8>) -> Result<(), OverflowError>{
        for value in values{
            self.memory[self.memory_pointer] = value;
            self.memory_pointer += 1;
            if self.memory_pointer >= self.memory.len() {
                return Err(OverflowError)
            }
        }
        Ok(())
    }

    fn handle_instruction(&mut self, instruction: &str, operands: &Option<Vec<String>>) -> Result<Vec<u8>, InvalidTokenError>{
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
        match self.calculate_expression(expr, offset, true)? {
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

    fn parse_positive_16bit_expr_immediately(&mut self, expr: &str) -> Result<u16, InvalidTokenError> {
        match self.calculate_expression(expr, 0, false)? {
            Some(v) if v>= 0 => {
                Ok(v as u16)
            }
            Some(_) => Err(InvalidTokenError { token: expr.into(), token_type: TokenType::Operand, additional_info: Some("Value cannot be nagative".into())}),
            None => Err(InvalidTokenError { token: expr.into(), token_type: TokenType::Operand, additional_info: Some("Expression cannot be immediately evaluated".into())}),
        }
    }

    fn parse_8bit_expr(&mut self, expr: &str, offset: usize) -> Result<u8, InvalidTokenError> {
        match self.calculate_expression(expr, offset,true)? {
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

    fn define_label(&mut self, name: &str) -> Result<(), InvalidTokenError> {
        let name = name.to_uppercase();
        self.validate_symbol_name(&name)?;

        if self.symbols.contains_key(&name){
            return Err(InvalidTokenError {token: name, token_type: TokenType::Label, additional_info: Some("Symbol already defined".into())})
        }

        self.symbols.insert(name, Symbol {value: self.memory_pointer as i32, kind: SymbolKind::Label});
        Ok(())
    }

    fn validate_symbol_name(&self, name: &str) -> Result<(), InvalidTokenError>{
        //We should allow labels with max 5 chars, but we will skip it for now
        if !name.is_ascii() {return Err(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Labels can only contain ASCII characters".into())})}

        let first_char = name.chars().next().ok_or(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Label is empty".into())})?;
        if !['@', '?', ':'].contains(&first_char) && !first_char.is_ascii_alphabetic() {return Err(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Labels cannot begin with a decimal digit or special character".into())});}

        if INSTRUCTIONS.contains(&name) || PSEUDO_INSTRUCTIONS.contains(&name){ return Err(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Labels cannot be the same as an instruction or a pseudo-instruction".into())});}

        Ok(())
    }

    fn assert_operand_amount(operands: &Option<Vec<String>>, allowed_amount: usize) -> Result<&Vec<String>, InvalidTokenError>{
        return match operands {
            Some(operands) => {
                if operands.len() < allowed_amount {
                    Err(InvalidTokenError { token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Too few operands".into()) })
                } else if operands.len() > allowed_amount {
                    Err(InvalidTokenError { token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Too many operands".into()) })
                } else {
                    Ok(operands)
                }
            }
            None => Err(InvalidTokenError { token: "".into(), token_type: TokenType::Operand, additional_info: Some("Too few operands".into()) })
        }
    }

    fn assert_valid_symbol_name(&self, name: &Option<String>) -> Result<String, InvalidTokenError>{
        let name = match name {
            Some(name) => name,
            None => return Err(InvalidTokenError { token: "".into(), token_type: TokenType::Label, additional_info: Some("Label is empty".into()) })
        };
        self.validate_symbol_name(name.as_str())?;
        Ok(name.clone())
    }

    fn handle_pseudo_instruction(&mut self, label: &Option<String>, instruction: &str, operands: &Option<Vec<String>>) -> Result<(), InvalidTokenError>{
        match instruction {
            "ORG" => {
                let operands = Self::assert_operand_amount(&operands,1)?;
                let address = self.parse_positive_16bit_expr_immediately(operands[0].as_str())?;
                self.memory_pointer = address as usize;
                Ok(())
            }
            "END" => {
                self.stopped = true;
                Ok(())
            }
            "EQU" => {
                let name = self.assert_valid_symbol_name(label)?;

                let operands = Self::assert_operand_amount(operands, 1)?;
                let value = self.calculate_expression(&operands[0], 0, false)?
                    .ok_or(InvalidTokenError {
                        token: operands[0].clone(),
                        token_type: TokenType::Operand,
                        additional_info: Some("EQU does not allow forward referencing".into()),
                    })?;

                if self.symbols.contains_key(&name) {
                    return Err(InvalidTokenError {
                        token: name.into(),
                        token_type: TokenType::Label,
                        additional_info: Some("Symbol with such name is already defined".into()),
                    });
                }

                self.symbols.insert(name, Symbol {
                    value,
                    kind: SymbolKind::Equ,
                });

                Ok(())
            }
            "SET" => {
                let name = self.assert_valid_symbol_name(label)?;

                let operands = Self::assert_operand_amount(operands, 1)?;
                let value = self.calculate_expression(&operands[0], 0, false)?
                    .ok_or(InvalidTokenError {
                        token: operands[0].clone(),
                        token_type: TokenType::Operand,
                        additional_info: Some("SET does not allow forward referencing".into()),
                    })?;

                match self.symbols.get(&name) {
                    None => {
                        self.symbols.insert(name, Symbol {
                            value,
                            kind: SymbolKind::Set,
                        });
                    }
                    Some(sym) if matches!(sym.kind, SymbolKind::Set) => {
                        self.symbols.insert(name, Symbol {
                            value,
                            kind: SymbolKind::Set,
                        });
                    }
                    Some(_) => {
                        return Err(InvalidTokenError {
                            token: name.to_string(),
                            token_type: TokenType::Label,
                            additional_info: Some("Cannot redefine non-SET symbol".into())
                        });
                    }
                }
                Ok(())
            }
            "MACRO" => {
                unimplemented!()
            }
            "ENDM" => {
                unimplemented!()
            }
            _ => Err( InvalidTokenError {token: instruction.into(), token_type:TokenType::Instruction, additional_info: Some("It is not a valid pseudo-instruction".into())})
        }
    }

    fn handle_if_instruction(&mut self, instruction: &str, operands: &Option<Vec<String>>) -> Result<(), InvalidTokenError>{
        let operands = Self::assert_operand_amount(operands, 1)?;
        let value = self.calculate_expression(&operands[0], 0, false)?
            .ok_or(InvalidTokenError {
                token: operands[0].clone(),
                token_type: TokenType::Operand,
                additional_info: Some("IF does not allow forward referencing".into()),
            })?;

        self.if_stack.push(value != 0);
        Ok(())
    }

    fn handle_endif_instruction(&mut self, instruction: &str, operands: &Option<Vec<String>>) -> Result<(), InvalidTokenError>{
        if self.if_stack.is_empty() {
            return Err(InvalidTokenError {
                token: "ENDIF".into(),
                token_type: TokenType::Instruction,
                additional_info: Some("ENDIF without matching IF".into()),
            });
        }
        self.if_stack.pop();
        Ok(())
    }

    fn handle_data_statement(&mut self, instruction: &str, operands: &Option<Vec<String>>) -> Result<Option<Vec<u8>>, TokenOrOverflowError>{
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
                                return Err(InvalidTokenError {token: operand.into(), token_type: TokenType::Operand, additional_info: Some("String contains non-ASCII characters".into())}.into())
                            }
                        }
                    } else {
                        values.push(self.parse_8bit_expr(operand, offset)?);
                        offset += 1;
                    }
                }
                Ok(Some(values))

            },
            "DW" => {
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
                    let (lo, hi) = self.parse_16bit_expr(operand, offset)?;
                    values.push(lo);
                    values.push(hi);
                    offset += 2;
                }
                Ok(Some(values))
            }
            "DS" => {
                let operands = Self::assert_operand_amount(operands, 1)?;
                let size = self.parse_positive_16bit_expr_immediately(operands[0].as_str())?;
                if self.memory_pointer + size as usize > self.memory.len() {
                    return Err(OverflowError.into())
                }
                self.memory_pointer += size as usize;
                Ok(None)
            },
            _ => Err( InvalidTokenError {token: instruction.into(), token_type:TokenType::Instruction, additional_info: Some("It is not a valid data statement".into())}.into())
        }
    }

    fn calculate_expression(
        &mut self,
        expr: &str,
        offset: usize,
        allow_forward_references: bool,
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
                if allow_forward_references {
                    self.pending_exprs.push(PendingExpr {
                        addr: self.memory_pointer + offset,
                        expr: ast,
                        line: self.current_line,
                    });
                } else {
                    return Err(InvalidTokenError {
                        token: expr.into(),
                        token_type: TokenType::Label,
                        additional_info: Some("Forward reference is not allowed".into()),
                    });
                }
                Ok(None)
            }
        }
    }

    fn tokenize(&self, expr: &str) -> Result<Vec<CalculationToken>, InvalidTokenError> {
        let pattern = r"(\bHERE\b|\$|\bMOD\b|\bNOT\b|\bAND\b|\bOR\b|\bXOR\b|\bSHL\b|\bSHR\b|\+|-|\*|/|\(|\))";
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
            tokens.push(CalculationToken::Num(v));
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
                self.symbols
                    .get(&l.to_uppercase())
                    .map(|s| s.value)
                    .ok_or(format!("Undefined symbol {}", l))
            }

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

    fn resolve_pending_exprs(&mut self) -> Result<(), InvalidTokenAtLineError> {
        for p in &self.pending_exprs {
            let v = self.eval_expr(&p.expr)
                .map_err(|e| InvalidTokenAtLineError {
                    line: p.line,
                    source: InvalidTokenError {
                        token: e,
                        token_type: TokenType::Label,
                        additional_info: None,
                    },
                })?;

            let b = v.to_le_bytes();
            self.memory[p.addr] = b[0];
            self.memory[p.addr + 1] = b[1];
        }
        Ok(())
    }
}