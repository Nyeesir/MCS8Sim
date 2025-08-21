use std::{error::Error, fmt, collections::HashMap};

const MEMORY_SIZE: usize = 65536;

#[derive(Debug, Clone)]
pub struct AssemblyError{
    line_number: usize,
    message: String
}

impl Error for AssemblyError {}
impl fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error in line {}: {}", self.line_number, self.message)
    }
}

#[derive(Debug, Clone)]
pub struct InvaildTokenError{
    token: String
}

impl Error for InvaildTokenError {}
impl fmt::Display for InvaildTokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invaild token: {}", self.token)
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
                None => return Err(AssemblyError{ line_number, message:"Non-empty line doesn't contain a word somehow".into()})
            };

            let instruction: &str;
            if token.contains(":") {
                match Self::add_jump_point(self, token) {
                    Ok(_) => {},
                    Err(e) => return Err(AssemblyError{ line_number, message:e.to_string()})
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
                Err(e) => return Err(AssemblyError{ line_number, message:e.to_string()})
            };

            for opcode in opcodes.iter(){
                self.memory[self.memory_pointer] = opcode.to_owned();
                self.memory_pointer += 1;
                if self.memory_pointer >= MEMORY_SIZE {
                    return Err(AssemblyError{ line_number, message: "Memory overflow".into()})
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
                let mut opcode: u8 = 0b00000100;
                let register = Self::translate_register(operands)?;
                opcode |= register << 3;
                opcodes.push(opcode);
            }
            "DCR" => {
                let mut opcode: u8 = 0b00000101;
                let register = Self::translate_register(operands)?;
                opcode |= register << 3;
                opcodes.push(opcode);
            }
            "CMA" => opcodes.push(0b00101111),
            "DAA" => opcodes.push(0b00100111),
            "NOP" => opcodes.push(0b00000000),
            "MOV" => {
                let mut opcode: u8 = 0b01000000;
                let (left_operand, right_operand) = match operands.split_once(","){
                    Some(x) => x,
                    None => return Err(InvaildTokenError{ token: operands.into()})
                };
                let left_register = Self::translate_register(left_operand)?;
                let right_register = Self::translate_register(right_operand)?;
                opcode |= (left_register << 3) & right_register;
                opcodes.push(opcode);
            }
            "STAX" => {
                let mut opcode: u8 = 0b00000010;
                match operands {
                    "BC" | "B" | "DE" | "D" => {}
                    _ => return Err(InvaildTokenError{ token: operands.into()})
                }
                let register_pair = Self::translate_register_pair(operands)?;
                opcode |= register_pair<<4;
                opcodes.push(opcode);
            }
            "LDAX" => {
                let mut opcode: u8 = 0b00001010;
                match operands {
                    "BC" | "B" | "DE" | "D" => {}
                    _ => return Err(InvaildTokenError{ token: operands.into()})
                }
                let register_pair = Self::translate_register_pair(operands)?;
                opcode |= register_pair<<4;
                opcodes.push(opcode);
            }
            "ADD" => {
                let mut opcode: u8 = 0b10000000;
                let register = Self::translate_register(operands)?;
                opcode |= register;
                opcodes.push(opcode);
            }
            "ADC" => {
                let mut opcode: u8 = 0b10001000;
                let register = Self::translate_register(operands)?;
                opcode |= register;
                opcodes.push(opcode);
            }
            "SUB" => {
                let mut opcode: u8 = 0b10010000;
                let register = Self::translate_register(operands)?;
                opcode |= register;
                opcodes.push(opcode);
            }
            "SBB" => {
                let mut opcode: u8 = 0b10011000;
                let register = Self::translate_register(operands)?;
                opcode |= register;
                opcodes.push(opcode);
            }
            "ANA" => {
                let mut opcode: u8 = 0b10100000;
                let register = Self::translate_register(operands)?;
                opcode |= register;
                opcodes.push(opcode);
            }
            "XRA" => {
                let mut opcode: u8 = 0b10101000;
                let register = Self::translate_register(operands)?;
                opcode |= register;
                opcodes.push(opcode);
            }
            "ORA" => {
                let mut opcode: u8 = 0b10110000;
                let register = Self::translate_register(operands)?;
                opcode |= register;
                opcodes.push(opcode);
            }
            "CMP" => {
                let mut opcode: u8 = 0b10111000;
                let register = Self::translate_register(operands)?;
                opcode |= register;
                opcodes.push(opcode);
            }
            "RLC" => opcodes.push(0b00000111),
            "RRC" => opcodes.push(0b00001111),
            "RAL" => opcodes.push(0b00010111),
            "RAR" => opcodes.push(0b00011111),
            "PUSH" => {
                let mut opcode: u8 = 0b11000101;
                let register_pair = Self::translate_register_pair(operands)?;
                opcode |= register_pair<<4;
                opcodes.push(opcode);
            }
            //TODO: Mozliwe ze trzeba dodac weryfikacje operandow tzn przyjmowac tylko psw albo sp w zaleznosci od instrukcji itd. Pewnie useless ale moze bedzie trzeba
            "POP" => {
                let mut opcode: u8 = 0b11000001;
                let register_pair = Self::translate_register_pair(operands)?;
                opcode |= register_pair<<4;
                opcodes.push(opcode);
            }
            "DAD" => {
                let mut opcode: u8 = 0b00001001;
                let register_pair = Self::translate_register_pair(operands)?;
                opcode |= register_pair<<4;
                opcodes.push(opcode);
            }
            "INX" => {
                let mut opcode: u8 = 0b00000011;
                let register_pair = Self::translate_register_pair(operands)?;
                opcode |= register_pair<<4;
                opcodes.push(opcode);
            }
            "DCX" => {
                let mut opcode: u8 = 0b00001011;
                let register_pair = Self::translate_register_pair(operands)?;
                opcode |= register_pair<<4;
                opcodes.push(opcode);
            }
            "XCHG" => opcodes.push(0b11101011),
            "XTHL" => opcodes.push(0b11100011),
            "SPHL" => opcodes.push(0b11111001),
            "MVI" => {
                
            }
            _ => return Err(InvaildTokenError{ token: instruction.into()})
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
            _ => Err(InvaildTokenError{ token: register.into()})
        }
    }

    fn translate_register_pair(register_pair: &str) -> Result<u8, InvaildTokenError>{
        match register_pair {
            "BC" | "B" => Ok(0b00),
            "DE" | "D" => Ok(0b01),
            "HL" | "H" => Ok(0b10),
            "SP" | "PSW" => Ok(0b11),
            _ => Err(InvaildTokenError{ token: register_pair.into()})
        }
    }

    fn _translate_label_or_address(_label_or_address: &str) -> Result<u16, InvaildTokenError>{
        Ok(0)
    }

    fn _translate_number(_number: &str) -> Result<u8, InvaildTokenError>{
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