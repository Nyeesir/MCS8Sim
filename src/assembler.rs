#[cfg(test)]
mod tests;

use std::{error::Error, fmt, collections::HashMap};
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
pub struct Assembler{
    memory: [u8; MEMORY_SIZE],
    memory_pointer: usize,
    jump_map: HashMap<String, usize>,
    missing_jumps: HashMap<usize, String>
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
        // match self.resolve_missing_jump_points(){
        //     Ok(_) => Ok(self.memory),
        //     Err(e) => Err(AssemblyError { line_number: 0, line_text: "".into(), message: e.to_string() })
        // }
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
                binary_values.push(Self::translate_value(&operand)?);
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
                binary_values.push(Self::translate_value(operands[0].as_str())?);
                //TODO: BRAKUJE PARSOWANIA WARTOSCI???
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
                match Self::parse_number_u8(operands[0].as_str()) {
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
                match Self::parse_number_u8(operands[0].as_str()) {
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
                match Self::parse_number_u8(register){
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
                match Self::parse_number_u8(register_pair){
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
        //For now, it's case-sensitive
        //TODO: do poprawy funkcja jak będzie ewaluacja wyrażeń
        if label_or_address == "$" {
            let address_bytes = self.memory_pointer.to_le_bytes();
            return [address_bytes[0], address_bytes[1]]
        }

        if self.jump_map.contains_key(label_or_address){
            let address_bytes = self.jump_map.get(label_or_address).unwrap().to_le_bytes();
            return [address_bytes[0], address_bytes[1]]
        }

        if let Ok(x) = u16::from_str_radix(&label_or_address, 10){
            return x.to_le_bytes()
        }
        let address_without_suffix = &label_or_address[0..label_or_address.len()-1];
        if label_or_address.ends_with("D"){
            if let Ok(x) = u16::from_str_radix(address_without_suffix, 10){return x.to_le_bytes()}
        }
        else if label_or_address.ends_with("B"){
            if let Ok(x) = u16::from_str_radix(address_without_suffix, 2){return x.to_le_bytes()}
        }
        else if label_or_address.ends_with("O") || label_or_address.ends_with("Q"){
            if let Ok(x) = u16::from_str_radix(address_without_suffix, 8){return x.to_le_bytes()}
        }
        else if label_or_address.ends_with("H"){
            if let Ok(x) = u16::from_str_radix(address_without_suffix, 16){return x.to_le_bytes()}
        }

        //Tutaj dodać zapisywanie pozycji do której będziemy wracać po skompilowaniu całego programu
        self.missing_jumps.insert(self.memory_pointer.wrapping_add(1), label_or_address.to_string());
        [0,0]

        // Err(InvalidTokenError{ token: address.into(), token_type: TokenType::Address, additional_info: Some("Only numeric values within u16 range with right suffixes or existing labels are allowed".into())})
    }


    fn translate_value(value: &str) -> Result<u8, InvalidTokenError>{
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

        match Self::parse_number_u8(value) {
            Ok(x) => Ok(x),
            Err(_) => Err(InvalidTokenError { token: value.into(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within u8 range with right suffixes or ASCII characters in single quotes are allowed".into()) })
        }
    }

    fn parse_number_i16(number: &str) -> Result<i16, InvalidTokenError>{
        let value = number.to_uppercase();
        if let Ok(x) = i16::from_str_radix(&value, 10){return Ok(x)}
        let value_without_suffix = &value[0..value.len()-1];
        if value.ends_with("D"){
            if let Ok(x) = i16::from_str_radix(value_without_suffix, 10){return Ok(x)}
        }
        else if value.ends_with("B"){
            if let Ok(x) = i16::from_str_radix(value_without_suffix, 2){return Ok(x)}
        }
        else if value.ends_with("O") || value.ends_with("Q"){
            if let Ok(x) = i16::from_str_radix(value_without_suffix, 8){return Ok(x)}
        }
        else if value.ends_with("H") && value.starts_with(&['-','0','1','2','3','4','5','6','7','8','9']){
            if let Ok(x) = i16::from_str_radix(value_without_suffix, 16){return Ok(x)}
        }
        Err(InvalidTokenError { token: value.into(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within valid range with right suffixes are allowed".into())})
    }

    fn parse_number_u8(number: &str) -> Result<u8, InvalidTokenError>{
        match Self::parse_number_i16(number){
            Ok(x) => {
                if x < 256 && x >= -128 {
                    Ok(x as u8)
                } else {
                    Err(InvalidTokenError { token: number.into(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within u8 range with right suffixes are allowed".into())})
                }
            }
            Err(_) => Err(InvalidTokenError { token: number.into(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within u8 range with right suffixes are allowed".into())})
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
        if !(['@', '?', ':'].contains(&first_char) || first_char.is_ascii_alphabetic()) {return Err(InvalidTokenError { token: label.into(), token_type: TokenType::Label, additional_info: Some("Labels cannot begin with a decimal digit".into())});}

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
                        values.push(Self::parse_number_u8(operand)?);
                    }
                    Ok(values)
                }
            },
            "DW" => unimplemented!(),
            "DS" => unimplemented!(),
            _ => Err( InvalidTokenError {token: instruction.into(), token_type:TokenType::Instruction, additional_info: Some("It is not a valid data statement".into())})
        }
    }

    fn calculate_expression(expression: &str) -> Result<u16, InvalidTokenError>{
        /*
        Operators cause expressions to be evaluated in the
        following order:
        1. Parenthesized expressions
        2. *,/, MOD, SHL, SHR
        3. +, - (unary and binary)
        4. NOT
        5. AND
        6. OR, XOR
         */


        //HERE to obecny adres
        //TODO: jako operandy do DB (może innych też) mogą być instrukcje
        //TODO: All operators treat their arguments as 15-bit quantities, and generate 16-bit quantities as their result.
        //TODO: Pamietac ze NOT jest unarny

        const OPERATIONS: [&str;13] = [ "MOD", "NOT", "AND", "OR", "XOR", "SHL", "SHR", "+", "-", "*","/", "(", ")"];
        const OPERATION_PRIORITY : [(&str,u16);11] = [("+",2),("-",2),("*",1),("/",1),(" MOD ",1),(" NOT ",3),(" AND ",4),(" OR ",5),(" XOR ",5),(" SHR ",1),(" SHL ",1)];
        if expression.matches("(").count() != expression.matches(")").count(){
            return Err(InvalidTokenError { token: expression.into(), token_type: TokenType::Operand, additional_info: Some("Parentheses are not balanced".into())})
        }

        let tokens = Self::split_expression(expression);
        let mut parsed_tokens: Vec<String> = Vec::new();

        //nie wiem czy jest na pewno zgodny z tym 15-bit quantities
        //pewnie lepiej by bylo to zrobic podczas wyliczania ostatecznej wartosci
        for token in tokens{
            if OPERATIONS.contains(&token.as_str()){
                parsed_tokens.push(token);
            } else {
                let token = Self::parse_number_i16(token.as_str())?.to_string();
                parsed_tokens.push(token);
            }
        }

        let infix_expression = Self::convert_infix_expr_to_postfix_expr(&parsed_tokens)?;


        unimplemented!()
        //TODO: dokonczyc
    }

    fn split_expression(expression: &str) -> Vec<String> {
        let pattern = r"( MOD |NOT | AND | OR | XOR | SHL | SHR |\+|\-|\*|/|\(|\))";
        let re = Regex::new(pattern).expect("Invalid regular expression");

        let mut result = Vec::new();
        let mut last = 0;

        for mat in re.find_iter(expression) {
            let start = mat.start();
            let end = mat.end();

            if start > last {
                result.push(expression[last..start].trim().to_string());
            }
            result.push(expression[start..end].trim().to_string());

            last = end;
        }

        if last < expression.len() {
            result.push(expression[last..].trim().to_string());
        }

        result.into_iter().filter(|s| !s.is_empty()).collect()
    }

    fn convert_infix_expr_to_postfix_expr(tokens: &Vec<String>) -> Result<Vec<String>, InvalidTokenError>{
        let priority = HashMap::from([("+",2),("-",2),("*",1),("/",1),("MOD",1),("NOT",3),("AND",4),("OR",5),("XOR",5),("SHR",1),("SHL",1)]);

        //NOT JEST ZLE TRAKTOWANY

        let mut output: Vec<String> = Vec::new();
        let mut stack: Vec<String> = Vec::new();

        for token in tokens {
            match token.as_str() {
                "(" => {
                    stack.push(token.clone());
                }
                ")" => {
                    while let Some(top) = stack.pop() {
                        if top == "(" {
                            break;
                        }
                        output.push(top);
                    }
                }

                op if priority.contains_key(op) => {
                    let p = priority[op];

                    while let Some(top) = stack.last() {
                        if top == "(" {
                            break;
                        }
                        if priority.contains_key(top.as_str()) {
                            let p_top = priority[top.as_str()];
                            if p_top <= p {
                                output.push(stack.pop().unwrap());
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    stack.push(op.to_string());
                }
                _ => {
                    output.push(token.clone());
                }
            }
        }

        while let Some(op) = stack.pop() {
            if op == "(" {
                return Err(InvalidTokenError {
                    token: "(".into(),
                    token_type: TokenType::Operand,
                    additional_info: Some("Unmatched '('".into()),
                });
            }
            output.push(op);
        }

        Ok(output)
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