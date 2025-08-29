use std::{error::Error, fmt, collections::HashMap};

const MEMORY_SIZE: usize = 65536;

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
            let line = line.trim().to_uppercase();
            if line.is_empty() {continue}

            let mut tokens_iter = line.split_whitespace();

            let token = match tokens_iter.next() {
                Some(x) => x,
                None => return Err(AssemblyError{ line_number, line_text: line, message:"Non-empty line doesn't contain a word somehow".into()})
            };

            let instruction: &str;
            if token.ends_with(":") {
                // TODO: walidacja nazw labeli
                /*
                    Here are some invalid label fields:
                        123: begins with a decimal digit
                        LABEL is not followed by a colon
                        ADD: is an operation code
                        END: is a pseudo-instruction
                 */

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
        //sprawdzić czy 'x' w ascii, potem normalne liczby a potem liczby z dopiskami wskazujacymi format liczby
        if value.len() == 3 && value.starts_with("\'") && value.ends_with("\'") {
            //puste ascii??
            let chars = value.chars().collect::<Vec<char>>();
            let ret: char = chars[1];
            if ret.is_ascii(){
                return Ok(ret as u8);
            }
        }
        //TODO: DOK
        if let Ok(x) = value.parse::<u8>(){}
        Ok(0)
    }

    fn add_jump_point(&mut self, label: &str) -> Result<(), DuplicateLabelError> {
        if self.jump_map.contains_key(label){
            return Err(DuplicateLabelError{ label: label.into()})
        }

        self.jump_map.insert(label.into(), self.memory_pointer);
        Ok(())
    }
}