#[cfg(test)]
mod assembler_tests;
pub(crate) mod errors;
mod expressions;
mod utils;
mod symbols;

use std::{collections::HashMap};
use std::iter::Peekable;
use std::str::Chars;
use errors::{AssemblyError, InvalidTokenError, OverflowError, TokenOrOverflowError, TokenType};
use expressions::PendingExpr;
use symbols::{Macro, MacroScope, Symbol, SymbolKind, SymbolScope};

/*
INS: DB (ADD C) nie jest obecnie możliwe, chyba do olania
DS i ORG nie działają do końca jak powinny przy operandach, nie obsługują forward referencing
W przypadku rejestrow nie przyjmujemy wyrażeń a tylko stałe w postaci odpowiednich stringów lub cyfr w przypadku pojedynczych rejestrów
Macra nie przyjmują komentarzy jako operandów
 */

//TODO: ZNAKI UCIECZKOWE W STRINGACH
//TODO: MOZELIWE ZE STRINGI OGRANICZYC DO 64 ZNAKOW



const MEMORY_SIZE: usize = u16::MAX as usize + 1;

pub const INSTRUCTIONS: [&str; 78] = ["STC", "CMC", "INR", "DCR", "CMA", "DAA", "NOP", "MOV", "STAX", "LDAX"
    , "ADD", "ADC", "SUB", "SBB", "ANA", "XRA", "ORA", "CMP", "RLC", "RRC", "RAL", "RAR", "PUSH"
    , "POP", "DAD", "INX", "DCX", "XCHG", "XTHL", "SPHL", "LXI", "MVI", "ADI", "ACI", "SUI", "SBI", "ANI"
    , "XRI", "ORI", "CPI", "STA", "LDA", "SHLD", "LHLD", "PCHL", "JMP", "JC", "JNC", "JZ", "JNZ", "JP", "JM", "JPE", "JPO"
    , "CALL", "CC", "CNC", "CZ", "CNZ", "CP", "CM", "CPE", "CPO", "RET", "RC", "RNC", "RZ", "RNZ", "RM", "RP", "RPE", "RPO"
    , "RST", "EI", "DI", "IN", "OUT", "HLT"];
pub const PSEUDO_INSTRUCTIONS: [&str; 8] = ["ORG", "EQU", "SET", "END", "IF", "ENDIF", "MACRO", "ENDM"];
pub const DATA_STATEMENTS: [&str; 3] = ["DB", "DW", "DS"];

pub struct Assembler{
    memory: [u8; MEMORY_SIZE],
    memory_pointer: usize,
    symbols: HashMap<String, Symbol>,
    macros: HashMap<String, Macro>,
    pending_exprs: Vec<PendingExpr>,
    stopped: bool,
    current_line: usize,
    if_stack: Vec<bool>,
    in_macro_definition: bool,
    current_macro_def_name: Option<String>,
    current_macro_scope: Option<MacroScope>,
    next_macro_expansion_id: u64,
    current_macro: Option<Macro>,
    in_macro_expansion: bool,
}

impl Assembler{
    pub fn new() -> Self{
        Assembler{
            memory: [0; MEMORY_SIZE],
            memory_pointer: 0,
            symbols: HashMap::new(),
            macros: HashMap::new(),
            pending_exprs: Vec::new(),
            stopped: false,
            current_line: 0,
            if_stack: Vec::new(),
            in_macro_definition: false,
            current_macro_def_name: None,
            current_macro_scope: None,
            next_macro_expansion_id: 0,
            current_macro: None,
            in_macro_expansion: false,
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

    fn parse_operands (char_iter: &mut Peekable<Chars>) -> Vec<String>{
        let mut is_inside_string = false;
        let mut field: String = String::new();
        let mut operands: Vec<String> = Vec::new();
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
        };

        //adding to an operand list remaining operand
        if !field.is_empty() {operands.push(field.trim().to_string())}

        operands
    }

    fn replace_param_token(line: &str, param: &str, value: &str) -> String {
        if param.is_empty() {
            return line.to_string();
        }

        let mut out = String::with_capacity(line.len());
        let mut token = String::new();
        let mut is_inside_string = false;

        for c in line.chars() {
            if c == '\'' {
                if !token.is_empty() {
                    if token == param {
                        out.push_str(value);
                    } else {
                        out.push_str(&token);
                    }
                    token.clear();
                }
                out.push(c);
                is_inside_string = !is_inside_string;
                continue;
            }

            if is_inside_string {
                out.push(c);
                continue;
            }

            if Self::is_ident_char(c) {
                token.push(c);
            } else {
                if !token.is_empty() {
                    if token == param {
                        out.push_str(value);
                    } else {
                        out.push_str(&token);
                    }
                    token.clear();
                }
                out.push(c);
            }
        }

        if !token.is_empty() {
            if token == param {
                out.push_str(value);
            } else {
                out.push_str(&token);
            }
        }

        out
    }

    fn is_ident_char(c: char) -> bool {
        matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '@' | '?')
    }


    fn fetch_fields(&self, line: &str) -> (Option<String>, Option<String>, Option<Vec<String>>){
        //RET label, instruction, operands; label and instruction are in upper case
        let mut ret = (None, None, None);
        let operands : Vec<String>;

        let mut line = line.trim();

        //removes comments
        if let Some((fields, _)) = line.split_once(";"){line = fields}
        if line.is_empty() { return ret }


        let mut char_iter = line.chars().peekable();
        let mut field;

        //parse first word
        field = Self::read_token_to_uppercase_to_nearest_space( &mut char_iter);

        if field.is_empty() { return ret }

        //check if the first word is an instruction, if not, it is a label
        if INSTRUCTIONS.contains(&field.as_str()) || PSEUDO_INSTRUCTIONS.contains(&field.as_str()) || DATA_STATEMENTS.contains(&field.as_str()) || self.macros.contains_key(&field) {
            ret.1 = Some(field.clone());
        } else {
            ret.0 = Some(field.clone());
        }
        field.clear();

        Self::advance_to_next_no_space_char(&mut char_iter);

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
        operands = Self::parse_operands(&mut char_iter);
        ret.2 = Some(operands);

        ret
    }

    fn is_compiling(&self) -> bool {
        self.if_stack.iter().all(|&v| v)
    }

    fn handle_fields(&mut self, label: &Option<String>, instruction: &Option<String>, operands: &Option<Vec<String>>) -> Result<(), TokenOrOverflowError>{
        if let Some(instruction) = instruction {
            if instruction == "IF" {
                self.handle_if_instruction(operands)?;
                return Ok(())
            }
            else if instruction == "ENDIF" {
                self.handle_endif_instruction()?;
                return Ok(())
            }
        }

        if !self.is_compiling() {
            return Ok(())
        }

        if let Some(label) = label {
            if label.contains(":") {
                Self::define_label(self, &label)?
            } else {
                match instruction.as_deref() {
                    Some("SET" | "EQU" | "MACRO") => {}
                    _ => {
                        return Err(
                            InvalidTokenError {token: label.clone(), token_type: TokenType::Label, additional_info: Some("Only SET, EQU and MACRO take labels without colons".into())}.into()
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

            match self.handle_macro_expansion(instruction, operands){
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

    fn handle_line(&mut self, line: &str, line_number: usize ) -> Result<(), AssemblyError>{
        self.current_line = line_number;
        let line = line.trim();
        if line.is_empty() { return Ok(()) }
        if !line.is_ascii() { return Err(AssemblyError { line_number, line_text: line.into(), message: "Non-ASCII characters found".into() })}

        let (label, instruction, operands) = Self::fetch_fields(self, &line);

        if self.in_macro_definition {
            if let Some(instruction) = instruction && instruction == "ENDM"{
                self.handle_endm_instruction().map_err(|e| AssemblyError { line_number, line_text: line.into(), message: e.to_string() })?;
            } else {
                self.current_macro
                    .as_mut()
                    .unwrap()
                    .body
                    .push(line.to_string());
            }
            return Ok(());
        }

        match self.handle_fields(&label, &instruction, &operands) {
            Ok(_) => {}
            Err(TokenOrOverflowError::Overflow(_)) => {
                return Err(AssemblyError { line_number, line_text: line.into(), message: "Overflow".into() })
            }
            Err(TokenOrOverflowError::InvalidToken(e)) => {
                return Err(AssemblyError { line_number, line_text: line.into(), message: e.to_string() })
            }
        }

        Ok(())
    }

    pub fn assemble (&mut self, data: &str) -> Result<[u8; MEMORY_SIZE], AssemblyError> {

        let mut script_lines = data.lines();
        let mut macro_lines: Option<std::vec::IntoIter<String>> = None;
        let mut script_line: usize = 0;

        while !self.stopped {
            let next_line: Option<(String, usize)> = if self.in_macro_expansion {
                if macro_lines.is_none() {
                    if let Some(current_macro) = self.current_macro.as_ref() {
                        macro_lines = Some(current_macro.body.clone().into_iter());
                    } else {
                        self.in_macro_expansion = false;
                        self.current_macro_scope = None;
                    }
                }

                if let Some(lines) = macro_lines.as_mut() {
                    if let Some(next_line) = lines.next() {
                        Some((next_line, script_line))
                    } else {
                        macro_lines = None;
                        self.in_macro_expansion = false;
                        self.current_macro = None;
                        self.current_macro_scope = None;
                        None
                    }
                } else {
                    None
                }
            } else if let Some(next_line) = script_lines.next() {
                script_line += 1;
                Some((next_line.to_string(), script_line))
            } else {
                self.stopped = true;
                None
            };

            if let Some((line, line_number)) = next_line {
                self.handle_line(&line, line_number)?;
            }
        }

        if !self.if_stack.is_empty() {
            return Err(AssemblyError {
                line_number: 0,
                line_text: "".to_string(),
                message: "Unclosed IF block".into(),
            });
        }

        if self.in_macro_definition {
            return Err(AssemblyError {
                line_number: 0,
                line_text: "".to_string(),
                message: "Unterminated MACRO definition".into(),
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
                    "BC" | "B" | "DE" | "D" | "H" | "HL" | "PSW" => {}
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
                    "IN" => binary_values[0] |= 0b0001000,
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
                let scope = self.current_macro_scope
                    .as_ref()
                    .map(|m| SymbolScope::Local(m.clone()))
                    .unwrap_or(SymbolScope::Global);
                let name = self.assert_valid_symbol_name(label, SymbolKind::Equ, &scope)?;

                let operands = Self::assert_operand_amount(operands, 1)?;
                let value = self.calculate_expression(&operands[0], 0, false)?
                    .ok_or(InvalidTokenError {
                        token: operands[0].clone(),
                        token_type: TokenType::Operand,
                        additional_info: Some("EQU does not allow forward referencing".into()),
                    })?;

                // self.symbols.insert(name, Symbol {value, kind: SymbolKind::Equ});
                self.set_symbol(name, value, SymbolKind::Equ);

                Ok(())
            }
            "SET" => {
                let scope = self.current_macro_scope
                    .as_ref()
                    .map(|m| SymbolScope::Local(m.clone()))
                    .unwrap_or(SymbolScope::Global);
                let name = self.assert_valid_symbol_name(label, SymbolKind::Set, &scope)?;

                let operands = Self::assert_operand_amount(operands, 1)?;
                let value = self.calculate_expression(&operands[0], 0, false)?
                    .ok_or(InvalidTokenError {
                        token: operands[0].clone(),
                        token_type: TokenType::Operand,
                        additional_info: Some("SET does not allow forward referencing".into()),
                    })?;

                // self.symbols.insert(name, Symbol {value, kind: SymbolKind::Set});
                self.set_symbol(name, value, SymbolKind::Set);
                Ok(())
            }
            "MACRO" => {
                Ok(self.handle_macro_instruction(label, operands)?)
            }
            _ => Err( InvalidTokenError {token: instruction.into(), token_type:TokenType::Instruction, additional_info: Some("It is not a valid pseudo-instruction".into())})
        }
    }

    fn handle_macro_instruction(&mut self, label: &Option<String>, operands: &Option<Vec<String>>) -> Result<(), InvalidTokenError>{
        if self.in_macro_definition {
            return Err(InvalidTokenError {
                token: "MACRO".into(),
                token_type: TokenType::Instruction,
                additional_info: Some("Nested MACRO definitions are not allowed".into()),
            });
        }

        let name = self.assert_valid_symbol_name(label, SymbolKind::Macro, &SymbolScope::Global)?;

        let mut params = Vec::new();
        if let Some(operands) = operands {
            for operand in operands {
                if self.validate_name(operand.as_str()).is_ok() {
                    params.push(operand.to_string());
                } else {
                    return Err(InvalidTokenError { token: operand.clone(), token_type: TokenType::Operand, additional_info: Some("Is not a valid parameter name".into())});
                }
            }
        }

        self.in_macro_definition = true;
        self.current_macro_def_name = Some(name.clone());
        self.current_macro = Some(Macro {
            params,
            body: Vec::new(),
        });

        Ok(())
    }

    fn handle_endm_instruction(&mut self) -> Result<(), InvalidTokenError>{
        if !self.in_macro_definition {
            return Err(InvalidTokenError {
                token: "ENDM".into(),
                token_type: TokenType::Instruction,
                additional_info: Some("ENDM without matching MACRO".into()),
            });
        }

        let name = self.current_macro_def_name.take().unwrap();
        let mac = self.current_macro.take().unwrap();

        self.macros.insert(name, mac);

        self.in_macro_definition = false;

        Ok(())
    }

    fn handle_if_instruction(&mut self, operands: &Option<Vec<String>>) -> Result<(), InvalidTokenError>{
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

    fn handle_endif_instruction(&mut self) -> Result<(), InvalidTokenError>{
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

    fn handle_macro_expansion(&mut self, instruction: &str, operands: &Option<Vec<String>>) -> Result<(), InvalidTokenError> {
        match self.macros.get(instruction) {
            Some(mac) => {
                let params_len = mac.params.len();
                let ops_len = operands.as_ref().map_or(0, |ops| ops.len());

                if params_len != ops_len {
                    return Err(InvalidTokenError {
                        token: instruction.into(),
                        token_type: TokenType::Operand,
                        additional_info: Some("Invalid number of macro operands".into()),
                    });
                }

                let mut expanded_macro = mac.clone();

                if params_len > 0 {
                    let mut lines: Vec<String> = Vec::new();
                    for origin_line in expanded_macro.body.iter() {
                        let mut line = origin_line.clone();
                        for i in 0..params_len {
                            line = Self::replace_param_token(
                                &line,
                                expanded_macro.params[i].as_str(),
                                operands.as_ref().unwrap()[i].as_str(),
                            );
                        }
                        lines.push(line);
                    }
                    expanded_macro.body = lines;
                }

                self.in_macro_expansion = true;
                self.current_macro_scope = Some(MacroScope {
                    name: instruction.to_string(),
                    id: self.next_macro_expansion_id,
                });
                self.next_macro_expansion_id += 1;
                self.current_macro = Some(expanded_macro);
            }
            None => {
                return Err( InvalidTokenError {token: instruction.into(), token_type:TokenType::Instruction, additional_info: Some("It is not a valid macro".into())}.into())
            }
        }

        Ok(())
    }
}
