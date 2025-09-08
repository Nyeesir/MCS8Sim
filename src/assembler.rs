use std::{error::Error, fmt, collections::HashMap};

//TODO: Dokonczyc normalne instrukcje, dodac instrukcje rezerwacji przestrzeni, macro

const MEMORY_SIZE: usize = 65536;
const INSTRUCTIONS: [&str; 107] = ["NOP", "LXI", "STAX", "INX", "INR", "DCR", "MVI", "RLC",
    "DSUB", "DAD", "LDAX", "DCX", "RRC",
    "ARHL", "RAL", "RDEL",
    "RIM", "SHLD", "DAA", "LDHI", "LHLD",
    "SIM", "LXI", "STA", "INX", "INR", "DCR", "MVI", "STC",
    "LDSI", "DAD", "LDA", "DCX", "INR", "DCR", "MVI", "CMC",
    "MOV", "HLT",
    "ADD", "ADC", "SUB", "SBB",
    "ANA", "XRA", "ORA", "CMP",
    "RNZ", "POP", "JNZ", "JMP", "CNZ", "PUSH", "ADI", "RST",
    "RZ", "RET", "JZ", "CZ", "CALL", "ACI",
    "RNC", "POP", "JNC", "OUT", "CNC", "PUSH", "SUI", "RST",
    "RC", "SHLX", "JC", "IN", "CC", "SBI", "RST",
    "RPO", "POP", "JPO", "XTHL", "CPO", "PUSH", "ANI", "RST",
    "RPE", "PCHL", "JPE", "XCHG", "CPE", "LHLX", "XRI", "RST",
    "RP", "POP", "JP", "DI", "CP", "PUSH", "ORI", "RST",
    "RM", "SPHL", "JM", "EI", "CM", "CPI", "RST"];
const PSEUDO_INSTRUCTIONS: [&str;8] = ["ORG", "EQU", "SET", "END", "IF", "END IF", "MACRO", "END M"];

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
pub struct InvaildTokenError{
    token: String,
    token_type: TokenType,
    additional_info: Option<String>
}

impl Error for InvaildTokenError {}
impl fmt::Display for InvaildTokenError {
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

#[derive(Debug, Clone)]
pub struct DuplicateLabelError{
    label: String
}

impl Error for DuplicateLabelError {}
impl fmt::Display for DuplicateLabelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Duplicate label: {}", self.label)
    }
}

pub struct Assembler{
    memory: [u8; MEMORY_SIZE],
    memory_pointer: usize,
    jump_map: HashMap<String, usize>
}

impl Assembler{
    pub fn new() -> Self{
        Assembler{
            memory: [0; MEMORY_SIZE],
            memory_pointer: 0,
            jump_map: HashMap::new()
        }
    }

    pub fn assemble (&mut self, data: &str) -> Result<[u8; MEMORY_SIZE], AssemblyError> {
        let mut line_number: usize = 0;

        let lines = data.lines();
        for line in lines{
            line_number += 1;
            let line = line.trim().to_owned();
            if line.is_empty() {continue}

            let mut tokens_iter = line.split_whitespace();

            let token = match tokens_iter.next() {
                Some(x) => x,
                None => return Err(AssemblyError{ line_number, line_text: line, message:"Non-empty line doesn't contain a word somehow".into()})
            };

            let instruction: &str;
            if token.ends_with(":") {
                match self.validate_label(token) {
                    Ok(_) => {},
                    Err(e) => return Err(AssemblyError{ line_number, line_text:line, message:e.to_string()})
                }
                match Self::add_jump_point(self, token) {
                    Ok(_) => {},
                    Err(e) => return Err(AssemblyError{ line_number, line_text:line, message:e.to_string()})
                }
                instruction = match tokens_iter.next() {
                    Some(x) => x,
                    None => continue
                };
            }
            else {
                instruction = token;
            }
            let operand = tokens_iter.next().unwrap_or_else(|| "");
            let opcodes = match Self::translate_instruction(instruction, operand) {
                Ok(x) => x,
                Err(e) => return Err(AssemblyError{ line_number, line_text:line, message:e.to_string()})
            };

            for opcode in opcodes.iter(){
                self.memory[self.memory_pointer] = opcode.to_owned();
                self.memory_pointer += 1;
                if self.memory_pointer >= MEMORY_SIZE {
                    return Err(AssemblyError{ line_number, line_text:line, message: "Memory overflow".into()})
                }
            }
        }

        Ok(self.memory)
    }

    fn translate_instruction(instruction: &str, operands: &str) -> Result<Vec<u8>, InvaildTokenError>{
        let instruction_in_upper = instruction.to_uppercase();
        let instruction = instruction_in_upper.as_str();
        //DATA STATEMENTS OMINALEM

        let mut opcodes: Vec<u8> = Vec::with_capacity(3);
        match instruction {
            "INR" => {
                opcodes.push(0b00000100);
                let register = Self::translate_register(operands)?;
                opcodes[0] |= register << 3;
            }
            "DCR" => {
                opcodes.push(0b00000101);
                let register = Self::translate_register(operands)?;
                opcodes[0] |= register << 3;
            }
            "CMA" => opcodes.push(0b00101111),
            "DAA" => opcodes.push(0b00100111),
            "NOP" => opcodes.push(0b00000000),
            "MOV" => {
                let (left_operand, right_operand) = match operands.split_once(","){
                    Some(x) => x,
                    None => return Err(InvaildTokenError{ token: operands.into(), token_type: TokenType::Operand, additional_info: None })
                };
                opcodes.push(0b01000000);
                let left_register = Self::translate_register(left_operand)?;
                let right_register = Self::translate_register(right_operand)?;
                opcodes[0] |= (left_register << 3) & right_register;
            }
            "STAX" => {
                match operands {
                    "BC" | "B" | "DE" | "D" => {}
                    _ => return Err(InvaildTokenError{ token: operands.into(), token_type: TokenType::Operand, additional_info: Some("Only BC, B, DE, D are allowed".into())})
                }
                opcodes.push(0b00000010);
                let register_pair = Self::translate_register_pair(operands)?;
                opcodes[0] |= register_pair<<4;
            }
            "LDAX" => {
                match operands {
                    "BC" | "B" | "DE" | "D" => {}
                    _ => return Err(InvaildTokenError{ token: operands.into(), token_type: TokenType::Operand, additional_info: Some("Only BC, B, DE, D are allowed".into())})
                }
                opcodes.push(0b00001010);
                let register_pair = Self::translate_register_pair(operands)?;
                opcodes[0] |= register_pair<<4;
            }
            "ADD" => {
                opcodes.push(0b10000000);
                let register = Self::translate_register(operands)?;
                opcodes[0] |= register;
            }
            "ADC" => {
                opcodes.push(0b10001000);
                let register = Self::translate_register(operands)?;
                opcodes[0] |= register;
            }
            "SUB" => {
                opcodes.push(0b10010000);
                let register = Self::translate_register(operands)?;
                opcodes[0] |= register;
            }
            "SBB" => {
                opcodes.push(0b10011000);
                let register = Self::translate_register(operands)?;
                opcodes[0] |= register;
            }
            "ANA" => {
                opcodes.push(0b10100000);
                let register = Self::translate_register(operands)?;
                opcodes[0] |= register;
            }
            "XRA" => {
                opcodes.push(0b10101000);
                let register = Self::translate_register(operands)?;
                opcodes[0] |= register;
            }
            "ORA" => {
                opcodes.push(0b10110000);
                let register = Self::translate_register(operands)?;
                opcodes[0] |= register;
            }
            "CMP" => {
                opcodes.push(0b10111000);
                let register = Self::translate_register(operands)?;
                opcodes[0] |= register;
            }
            "RLC" => opcodes.push(0b00000111),
            "RRC" => opcodes.push(0b00001111),
            "RAL" => opcodes.push(0b00010111),
            "RAR" => opcodes.push(0b00011111),
            "PUSH" => {
                opcodes.push(0b11000101);
                let register_pair = Self::translate_register_pair(operands)?;
                opcodes[0] |= register_pair<<4;
            }
            //TODO: Mozliwe ze trzeba dodac weryfikacje operandow tzn przyjmowac tylko psw albo sp w zaleznosci od instrukcji itd. Pewnie useless ale moze bedzie trzeba
            "POP" => {
                opcodes.push(0b11000001);
                let register_pair = Self::translate_register_pair(operands)?;
                opcodes[0] |= register_pair<<4;
            }
            "DAD" => {
                opcodes.push(0b00001001);
                let register_pair = Self::translate_register_pair(operands)?;
                opcodes[0] |= register_pair<<4;
            }
            "INX" => {
                opcodes.push(0b00000011);
                let register_pair = Self::translate_register_pair(operands)?;
                opcodes[0] |= register_pair<<4;
            }
            "DCX" => {
                opcodes.push(0b00001011);
                let register_pair = Self::translate_register_pair(operands)?;
                opcodes[0] |= register_pair<<4;
            }
            "XCHG" => opcodes.push(0b11101011),
            "XTHL" => opcodes.push(0b11100011),
            "SPHL" => opcodes.push(0b11111001),
            "MVI" => {
                opcodes.push(0b00000110);
                let (left_operand, right_operand) = match operands.split_once(","){
                    Some(x) => x,
                    None => return Err(InvaildTokenError{ token: operands.into(), token_type: TokenType::Operand, additional_info: None})
                };
                let register = Self::translate_register(left_operand)?;
                opcodes[0] |= register << 3;
                let value = Self::translate_value(right_operand)?;
                opcodes.push(value);
            }
            _ => return Err(InvaildTokenError{ token: instruction.into(), token_type: TokenType::Instruction, additional_info: None})
        }
        Ok(opcodes)
    }

    fn translate_register(register: &str) -> Result<u8, InvaildTokenError>{
        let register_in_upper = register.to_uppercase();
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
            _ => Err(InvaildTokenError{ token: register.into(), token_type: TokenType::Operand, additional_info: Some("Only registers are allowed".into())})
        }
    }

    fn translate_register_pair(register_pair: &str) -> Result<u8, InvaildTokenError>{
        let register_pair_in_upper = register_pair.to_uppercase();
        let register_pair = register_pair_in_upper.as_str();
        match register_pair {
            "BC" | "B" => Ok(0b00),
            "DE" | "D" => Ok(0b01),
            "HL" | "H" => Ok(0b10),
            "SP" | "PSW" => Ok(0b11),
            _ => Err(InvaildTokenError{ token: register_pair.into(), token_type: TokenType::Operand, additional_info: Some("Only register pairs are allowed".into())})
        }
    }

    fn _translate_label_or_address(_label_or_address: &str) -> Result<u16, InvaildTokenError>{
        Ok(0)
    }

    fn translate_value(value: &str) -> Result<u8, InvaildTokenError>{
        if (value.len() == 3 || value.len() == 2) && value.starts_with("\'") && value.ends_with("\'") {
            if value.len() == 2 {
                return Ok(0)
            }
            let chars = value.chars().collect::<Vec<char>>();
            let ret: char = chars[1];
            return if ret.is_ascii() {
                Ok(ret as u8)
            } else {
                Err(InvaildTokenError { token: value.into(), token_type: TokenType::Operand, additional_info: Some("Only ASCII characters are allowed".into()) })
            }
        }

        let value = value.to_uppercase();
        if let Ok(x) = u8::from_str_radix(&value, 10){return Ok(x)}
        if value.ends_with("D"){
            if let Ok(x) = u8::from_str_radix(&value[0..value.len()-1], 10){return Ok(x)}
        }
        else if value.ends_with("B"){
            if let Ok(x) = u8::from_str_radix(&value[0..value.len()-1], 2){return Ok(x)}
        }
        else if value.ends_with("O") || value.ends_with("Q"){
            if let Ok(x) = u8::from_str_radix(&value[0..value.len()-1], 8){return Ok(x)}
        }
        else if value.ends_with("H"){
            if let Ok(x) = u8::from_str_radix(&value[0..value.len()-1], 16){return Ok(x)}
        }
        Err(InvaildTokenError{ token: value.into(), token_type: TokenType::Operand, additional_info: Some("Only numeric values with right suffixes or ASCII characters in single quotes are allowed".into())})
    }

    fn add_jump_point(&mut self, label: &str) -> Result<(), DuplicateLabelError> {
        if self.jump_map.contains_key(label){
            return Err(DuplicateLabelError{ label: label.into()})
        }

        self.jump_map.insert(label.into(), self.memory_pointer);
        Ok(())
    }

    fn validate_label(&self, label: &str) -> Result<(), InvaildTokenError>{
        let label_to_upper = label.to_uppercase();
        let label = label_to_upper.as_str();
        /*
        Has to be ASCII
        Here are some invalid label fields:
        123: begins with a decimal digit
        LABEL is not followed by a colon
        ADD: is an operation code
        END: is a pseudo-instruction

        */
        let label = &label[0..label.len()-1];
        if !label.is_ascii() {return Err(InvaildTokenError{ token: label.into(), token_type: TokenType::Label, additional_info: Some("Labels can only contain ASCII characters".into())})}

        let first_char = label.chars().next().ok_or(InvaildTokenError{ token: label.into(), token_type: TokenType::Label, additional_info: Some("Label is empty".into())})?;
        if first_char.is_ascii_digit(){ return Err(InvaildTokenError{ token: label.into(), token_type: TokenType::Label, additional_info: Some("Labels cannot begin with a decimal digit".into())}); }

        if INSTRUCTIONS.contains(&label) || PSEUDO_INSTRUCTIONS.contains(&label){ return Err(InvaildTokenError{ token: label.into(), token_type: TokenType::Label, additional_info: Some("Labels cannot be the same as an instruction or a pseudo-instruction".into())});}

        Ok(())
        }
}