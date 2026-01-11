#[cfg(test)]
mod assembler_tests;

use std::{error::Error, fmt, collections::HashMap};
use std::iter::Peekable;
use std::slice::Iter;
use egui::TextBuffer;
use regex::Regex;
//TODO: Dodac instrukcje rezerwacji przestrzeni, macro
//TODO: Dodac ewaluacje wyrazen arytmetycznych i logicznych jako operandow (strona 10) i dostosowac do tego parsowanie tokenow
//TODO: Dodac zmienne przechowujace start i koniec programu


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
enum TokenType{
    Instruction,
    Operand,
    Label,
    Address,
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
            TokenType::Address => error_message = "Invalid address".into(),
        }
        error_message.push_str(&format!(": {}.", self.token));
        if let Some(x) = &self.additional_info {
            error_message.push_str(&format!("Additional info: {}", x));
        }
        write!(f, "{}", error_message)
    }
}

#[derive(Clone)]
pub struct Assembler{
    memory: [u8; MEMORY_SIZE],
    memory_pointer: usize,
    jump_map: HashMap<String, usize>,
    missing_jumps: HashMap<usize, String>
}

#[derive(Debug, Clone)]
enum Tok {
    Num(u16),
    Op(String),
    LParen,
    RParen,
}

impl Assembler{
    pub fn new() -> Self{
        Assembler{
            memory: [0; MEMORY_SIZE],
            memory_pointer: 0,
            jump_map: HashMap::new(),
            missing_jumps: HashMap::new()
        }
    }

    pub fn assemble (&mut self, data: &str) -> Result<[u8; MEMORY_SIZE], AssemblyError> {
        let mut line_number: usize = 0;

        let lines = data.lines();
        for line in lines{
            line_number += 1;
            let line = line.trim().to_owned();
            if line.is_empty() {continue}

            let mut instruction: String = String::new();
            let mut label: String = String::new();
            let mut operands : Vec<String> = Vec::new();

            //replace multiple spaces with a single space
            let line = line.split_whitespace().collect::<Vec<&str>>().join(" ");

            let mut word: String = String::new();
            let mut char_iter = line.chars().into_iter();

            //parsing label or instruction
            while let Some(char) = char_iter.next() {
                if char.is_whitespace(){
                    break;
                } else {
                    word.push(char);
                }
            }
            if !word.is_empty() {
                if word.ends_with(":"){
                    label = word.clone();
                } else {
                    instruction = word.clone();
                }
                word.clear();
            } else {
                continue;
            }

            //handling label if present
            if !label.is_empty() {
                match Self::add_jump_point(self, &label) {
                    Ok(_) => {},
                    Err(e) => return Err(AssemblyError { line_number, line_text: line, message: e.to_string() })
                }
            }

            //parsing instruction if the first token was a label
            if instruction.is_empty() {
                while let Some(char) = char_iter.next() {
                    if char.is_whitespace(){
                        break;
                    } else {
                        word.push(char);
                    }
                }
                if !word.is_empty() {
                    instruction = word.clone();
                    word.clear();
                } else {
                    continue
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
            }
            if !word.is_empty() {operands.push(word.trim().to_owned())}

            match Self::handle_data_statement(&instruction, &operands) {
                Ok(binary_values) => {
                    Self::save_values_to_memory(self, binary_values)?;
                    continue;
                },
                Err(_) => {}
            }
            // if Self::handle_pseudo_instruction(self, label, instruction, &operands).is_ok() {continue}
            // if Self::handle_macro().is_ok() {continue}


            if !instruction.is_empty() {
                let binary_values = match Self::translate_instruction(self, &instruction, &operands) {
                    Ok(x) => x,
                    Err(e) => return Err(AssemblyError { line_number, line_text: line, message: e.to_string() })
                };

                Self::save_values_to_memory(self, binary_values)?;
            }
        }
        self.resolve_missing_jump_points().or(Err(AssemblyError { line_number: 0, line_text: "".into(), message: "Could not resolve missing jump points".into() }))?;
        Ok(self.memory)
    }

    fn save_values_to_memory(&mut self, values: Vec<u8>) -> Result<(), AssemblyError>{
        for value in values{
            self.memory[self.memory_pointer] = value;
            self.memory_pointer += 1;
            if self.memory_pointer >= MEMORY_SIZE {
                return Err(AssemblyError { line_number: 0, line_text: "".into(), message: "Memory overflow".into() })
            }
        }
        Ok(())
    }

    fn translate_instruction(&mut self, instruction: &str, operands: &Vec<String>) -> Result<Vec<u8>, InvalidTokenError>{
        let instruction_in_upper = instruction.to_uppercase();
        let instruction = instruction_in_upper.as_str();

        let mut binary_values: Vec<u8> = Vec::with_capacity(3);
        match instruction {
            "STC" => binary_values.push(0b00110111),
            "CMC" => binary_values.push(0b00111111),
            "INR" => {
                binary_values.push(0b00000100);
                Self::assert_operand_amount(operands, 1)?;
                let register = Self::parse_register(operands[0].as_str())?;
                binary_values[0] |= register << 3;
            }
            "DCR" => {
                binary_values.push(0b00000101);
                Self::assert_operand_amount(operands, 1)?;
                let register = Self::parse_register(operands[0].as_str())?;
                binary_values[0] |= register << 3;
            }
            "CMA" => binary_values.push(0b00101111),
            "DAA" => binary_values.push(0b00100111),
            "NOP" => binary_values.push(0b00000000),
            "MOV" => {
                binary_values.push(0b01000000);
                Self::assert_operand_amount(operands, 2)?;
                let (left_register, right_register) = (Self::parse_register(operands[0].as_str())?, Self::parse_register(operands[1].as_str())?);
                binary_values[0] |= (left_register << 3) | right_register;
            }
            "STAX" | "LDAX" => {
                Self::assert_operand_amount(operands, 1)?;
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
                Self::assert_operand_amount(operands, 1)?;
                let register = Self::parse_register(operands[0].as_str())?;
                binary_values[0] |= register;
            }
            "RLC" => binary_values.push(0b00000111),
            "RRC" => binary_values.push(0b00001111),
            "RAL" => binary_values.push(0b00010111),
            "RAR" => binary_values.push(0b00011111),
            "PUSH" => {
                binary_values.push(0b11000101);
                Self::assert_operand_amount(operands, 1)?;
                let register_pair = Self::parse_register_pair(operands[0].as_str())?;
                binary_values[0] |= register_pair<<4;
            }
            "POP" => {
                binary_values.push(0b11000001);
                Self::assert_operand_amount(operands, 1)?;
                let register_pair = Self::parse_register_pair(operands[0].as_str())?;
                match operands[0].as_str() {
                    "Bc" | "B" | "DE" | "D" | "H" | "HL" | "PSW" => {}
                    _ => return Err(InvalidTokenError { token: operands[0].clone(), token_type: TokenType::Operand, additional_info: Some("Only BC, B, DE, D are allowed".into())})
                }
                binary_values[0] |= register_pair<<4;
            }
            "DAD" => {
                binary_values.push(0b00001001);
                Self::assert_operand_amount(operands, 1)?;
                let register_pair = Self::parse_register_pair(operands[0].as_str())?;
                binary_values[0] |= register_pair<<4;
            }
            "INX" => {
                binary_values.push(0b00000011);
                Self::assert_operand_amount(operands, 1)?;
                let register_pair = Self::parse_register_pair(operands[0].as_str())?;
                binary_values[0] |= register_pair<<4;
            }
            "DCX" => {
                binary_values.push(0b00001011);
                Self::assert_operand_amount(operands, 1)?;
                let register_pair = Self::parse_register_pair(operands[0].as_str())?;
                binary_values[0] |= register_pair<<4;
            }
            "XCHG" => binary_values.push(0b11101011),
            "XTHL" => binary_values.push(0b11100011),
            "SPHL" => binary_values.push(0b11111001),
            "LXI" => {
                binary_values.push(0b00000001);
                Self::assert_operand_amount(operands, 2)?;
                let (register_pair, operand) = (operands[0].as_str(), operands[1].as_str());
                let register_pair = Self::parse_register_pair(register_pair)?;
                binary_values[0] |= register_pair << 4;
                for value in self.parse_label_or_address(operand){
                    binary_values.push(value);
                }
            }
            "MVI" => {
                binary_values.push(0b00000110);
                Self::assert_operand_amount(operands, 2)?;
                let (register, operand) = (operands[0].as_str(), operands[1].as_str());
                let register = Self::parse_register(register)?;
                binary_values[0] |= register << 3;
                binary_values.push(Self::translate_8bit_value(&operand)?);
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
                Self::assert_operand_amount(operands, 1)?;
                binary_values.push(Self::translate_8bit_value(operands[0].as_str())?);
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
                Self::assert_operand_amount(operands, 1)?;
                for value in self.parse_label_or_address(&operands[0]){
                    binary_values.push(value);
                }
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
                Self::assert_operand_amount(operands, 1)?;
                for value in self.parse_label_or_address(&operands[0]){
                    binary_values.push(value);
                }
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
                Self::assert_operand_amount(operands, 1)?;
                for value in self.parse_label_or_address(&operands[0]){
                        binary_values.push(value);
                }
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
                Self::assert_operand_amount(operands, 1)?;
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
                Self::assert_operand_amount(&operands, 1)?;
                match Self::parse_8bit_number(operands[0].as_str()) {
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
                match Self::parse_8bit_number(register_pair){
                    Ok(x) => {
                        if x < 4 {
                            Ok(x)
                        } else { Err(InvalidTokenError { token: register_pair.into(), token_type: TokenType::Operand, additional_info: Some("Register pair number is out of range".into())}) }
                    }
                    Err(_) => Err(InvalidTokenError { token: register_pair.into(), token_type: TokenType::Operand, additional_info: Some("Only register pairs as words or their numeric presentation is allowed".into())})
                }
            }
        }
    }

    fn parse_label_or_address(&mut self, label_or_address: &str) -> [u8;2]{
        let label_or_address = label_or_address.to_uppercase();
        let label_or_address = label_or_address.as_str();
        //TODO: add relative addresses with dolar sign
        //TODO: do poprawy funkcja jak będzie ewaluacja wyrażeń
        if label_or_address == "$" {
            let address_bytes = self.memory_pointer.to_le_bytes();
            return [address_bytes[0], address_bytes[1]]
        }

        if self.jump_map.contains_key(label_or_address){
            let address_bytes = self.jump_map.get(label_or_address).unwrap().to_le_bytes();
            return [address_bytes[0], address_bytes[1]]
        }

        //Zastapic parsem
        // if let Ok(x) = u16::from_str_radix(&label_or_address, 10){
        //     return x.to_le_bytes()
        // }
        // let address_without_suffix = &label_or_address[0..label_or_address.len()-1];
        // if label_or_address.ends_with("D"){
        //     if let Ok(x) = u16::from_str_radix(address_without_suffix, 10){return x.to_le_bytes()}
        // }
        // else if label_or_address.ends_with("B"){
        //     if let Ok(x) = u16::from_str_radix(address_without_suffix, 2){return x.to_le_bytes()}
        // }
        // else if label_or_address.ends_with("O") || label_or_address.ends_with("Q"){
        //     if let Ok(x) = u16::from_str_radix(address_without_suffix, 8){return x.to_le_bytes()}
        // }
        // else if label_or_address.ends_with("H") && label_or_address.starts_with(&['-','0','1','2','3','4','5','6','7','8','9']){
        //     if let Ok(x) = u16::from_str_radix(address_without_suffix, 16){return x.to_le_bytes()}
        // }
        if let Ok(x) = Self::parse_16bit_number(&label_or_address) {
            return x.to_le_bytes();
        }

        //Tutaj dodać zapisywanie pozycji do której będziemy wracać po skompilowaniu całego programu
        self.missing_jumps.insert(self.memory_pointer.wrapping_add(1), label_or_address.to_string());
        [0,0]
    }


    fn translate_8bit_value(value: &str) -> Result<u8, InvalidTokenError>{
        if (value.len() == 3 || value.len() == 2) && value.starts_with("'") && value.ends_with("'") {
            if value.len() == 2 {
                return Ok(0)
            }
            let chars = value.chars().collect::<Vec<char>>();
            let ret: char = chars[1];
            return if ret.is_ascii() {
                Ok(ret as u8)
            } else {
                Err(InvalidTokenError { token: value.into(), token_type: TokenType::Operand, additional_info: Some("Only ASCII characters are allowed".into()) })
            }
        }

        match Self::parse_8bit_number(value) {
            Ok(x) => Ok(x),
            Err(_) => Err(InvalidTokenError { token: value.into(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within u8 range with right suffixes or ASCII characters in single quotes are allowed".into()) })
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

    fn parse_16bit_number(number: &str) -> Result<u16, InvalidTokenError>{
        match Self::parse_number_i32(number){
            Ok(x) => {
                let v = x as i32;
                if (i16::MIN as i32..= u16::MAX as i32).contains(&v) {
                    Ok(v as u16)
                } else {
                    Err(InvalidTokenError { token: number.into(), token_type: TokenType::Operand, additional_info: Some("Only 16-bit numeric values with right suffixes are allowed".into())})
                }
            }
            Err(_) => Err(InvalidTokenError { token: number.into(), token_type: TokenType::Operand, additional_info: Some("Only 16-bit numeric values with right suffixes are allowed".into())})
        }
    }

    fn add_jump_point(&mut self, label: &str) -> Result<(), InvalidTokenError> {
        let label = label.trim().to_uppercase();;
        let label = &label[0..label.len()-1];

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

    fn assert_operand_amount(operands: &Vec<String>, allowed_amount: usize) -> Result<(), InvalidTokenError>{
        if operands.len() < allowed_amount{
            return Err(InvalidTokenError { token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Too less operands".into())})
        } else if operands.len() > allowed_amount{
            return Err(InvalidTokenError { token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Too many operands".into())})
        }
        Ok(())
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

    fn handle_data_statement(instruction: &str, operands: &Vec<String>) -> Result<Vec<u8>, InvalidTokenError>{
        let instruction_in_upper = instruction.to_uppercase();
        let instruction = instruction_in_upper.as_str();

        let mut values = Vec::new();
        match instruction {
            "DB" => {
                //String in single quotes handling
                if operands.len() == 1 && operands[0].starts_with("'") && operands[0].ends_with("'"){
                    for char in operands[0].chars(){
                        if char.is_ascii(){
                            values.push(char as u8);
                        }
                    }
                    Ok(values)
                }
                else {
                    //TODO: DALEJ NIE DZIALAJA WYRAZENIA ARYTMETYCZNO LOGICZNE
                    for operand in operands{
                        values.push(Self::parse_8bit_number(operand)?);
                    }
                    Ok(values)
                }
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
    */

    fn calculate_expression_to_8bit(self, expr: &str) -> Result<u8, InvalidTokenError> {
        match self.calculate_expression(expr) {
            Ok(x) => {
                let v = x as i16;
                if (i8::MIN as i16..= u8::MAX as i16).contains(&v) {
                    Ok(v as u8)
                } else {
                    Err(InvalidTokenError {
                        token: expr.into(),
                        token_type: TokenType::Operand,
                        additional_info: Some("Expression result is out of 8-bit range".into()),
                    })
                }
            }
            Err(e) => Err(e),
        }
    }

    fn calculate_expression_to_16bit(self, expr: &str) -> Result<u16, InvalidTokenError> {
        //TODO:Upewnic sie ze dziala jak nalezy
        match self.calculate_expression(expr) {
            Ok(x) => {Ok(x)}
            Err(e) => {Err(e)}
        }
    }


    fn calculate_expression(self, expr: &str) -> Result<u16, InvalidTokenError> {
        let tokens = Self::tokenize(self, expr)?;
        let mut it = tokens.iter().peekable();

        let value = match Self::parse_expr(&mut it, 0) {
            Ok(x) => x,
            Err(mut e) => {
                e.token = expr.into();
                return Err(e);
            }
        };

        if it.peek().is_some() {
            return Err(InvalidTokenError{token: expr.into(), token_type:TokenType::Operand, additional_info:Some("Unexpected token at end of expression".into())});
        }

        Ok(value)
    }


    fn tokenize(self, expr: &str) -> Result<Vec<Tok>, InvalidTokenError> {
        let pattern = r"(HERE|\$|MOD|NOT|AND|OR|XOR|SHL|SHR|\+|\-|\*|/|\(|\))";
        let re = Regex::new(pattern).unwrap();

        let mut tokens = Vec::new();
        let mut last = 0;

        for m in re.find_iter(expr) {
            if m.start() > last {
                let part = expr[last..m.start()].trim();
                if !part.is_empty() {
                    let v = Self::parse_number_i32(part).map_err(|_| {
                        InvalidTokenError{token: expr.into(), token_type:TokenType::Operand, additional_info:Some(format!("Invalid number: {}", part))}
                    })?;
                    tokens.push(Tok::Num(v as u16));
                }
            }

            let t = m.as_str();
            tokens.push(match t {
                "(" => Tok::LParen,
                ")" => Tok::RParen,
                "HERE" | "$" => Tok::Num(self.memory_pointer as u16),
                _ => Tok::Op(t.to_string()),
            });

            last = m.end();
        }

        if last < expr.len() {
            let part = expr[last..].trim();
            if !part.is_empty() {
                let v =  Self::parse_number_i32(part).map_err(|_| {
                    InvalidTokenError{token: expr.into(), token_type:TokenType::Operand, additional_info:Some(format!("Invalid number: {}", part))}
                })?;
                tokens.push(Tok::Num(v as u16));
            }
        }

        Ok(tokens)
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
        tokens: &mut Peekable<Iter<Tok>>,
        min_prec: u8,
    ) -> Result<u16, InvalidTokenError> {

        let mut lhs = match tokens.next() {
            Some(Tok::Num(v)) => *v,

            Some(Tok::Op(op)) if op == "-" => {
                //TODO: ZASTANOWIC SIE NAD PRIORYTETEM
                let v = Self::parse_expr(tokens, Self::precedence("NOT"))?;
                (!v).wrapping_add(1) & 0xFFFF
            }

            Some(Tok::Op(op)) if op == "NOT" => {
                let v = Self::parse_expr(tokens, Self::precedence("NOT"))?;
                !v & 0xFFFF
            }

            Some(Tok::LParen) => {
                let v = Self::parse_expr(tokens, 0)?;
                match tokens.next() {
                    Some(Tok::RParen) => v,
                    _ => return Err(InvalidTokenError{token: "".into(), token_type:TokenType::Operand, additional_info:Some("Missing \")\"".into())}),
                }
            }

            _ => return Err(InvalidTokenError{token: "".into(), token_type:TokenType::Operand, additional_info:Some("Expected operator".into())}),
        };

        while let Some(Tok::Op(op)) = tokens.peek() {
            let prec = Self::precedence(op);
            if prec < min_prec {
                break;
            }

            let op = match tokens.next() {
                Some(Tok::Op(o)) => o.clone(),
                _ => unreachable!(),
            };

            let rhs = Self::parse_expr(tokens, prec + 1)?;
            lhs = Self::eval_bin(&op, lhs, rhs);
        }

        Ok(lhs)
    }

    fn eval_bin(op: &str, a: u16, b: u16) -> u16 {
        let r = match op {
            "+" => a.wrapping_add(b),
            "-" => a.wrapping_sub(b),
            "*" => a.wrapping_mul(b),
            "/" => a / b,
            "MOD" => a % b,
            "AND" => a & b,
            "OR"  => a | b,
            "XOR" => a ^ b,
            "SHL" => a << b,
            "SHR" => a >> b,
            _ => unreachable!(),
        };
        r & 0xFFFF
    }

    fn resolve_missing_jump_points(&mut self) -> Result<(), InvalidTokenError>{
        //FIXME: trzeba jakoś ładnie przekazywać wartości żeby podczas wyrzucania błędów je wyświetlać; nowy typ błędu albo coś

        for (memory_pointer, label) in &self.missing_jumps{
            let address = match self.jump_map.get(label){
                Some(x) => x,
                None => return Err(InvalidTokenError { token: label.into(), token_type: TokenType::Label, additional_info: Some("Label does not exist".into())})
            };
            let address_bytes = address.to_le_bytes();
            self.memory[*memory_pointer] = address_bytes[0];
            self.memory[memory_pointer+1] = address_bytes[1];
        }
        Ok(())
    }
}