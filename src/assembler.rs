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
    pub fn assemble (&mut self, data: &str) -> Result<[u8; MEMORY_SIZE], AssemblyError> {
        let mut line_number: usize = 0;

        let lines = data.lines();
        for line in lines{
            line_number += 1;
            let line = line.trim().to_uppercase();
            if line.is_empty() {continue}

            let mut tokens_iter = line.split_whitespace();

            let token = match (tokens_iter.next()) {
                Some(x) => x,
                None => return Err(AssemblyError{ line_number, message:"Non-empty line doesn't contain a word somehow".into()})
            };

            let instruction: &str;
            if token.contains(":") {
                match (Self::add_jump_point(self,token)) {
                    Ok(_) => {},
                    Err(e) => return Err(AssemblyError{ line_number, message:e.to_string()})
                }
                instruction = match(tokens_iter.next()) {
                    Some(x) => x,
                    None => continue
                };
            }
            else {
                instruction = token;
            }
            let operand = (tokens_iter.next()).unwrap_or_else(|| "");
            let opcodes = match (Self::translate_instruction(instruction, operand)){
                Ok(x) => x,
                Err(e) => return Err(AssemblyError{ line_number, message:e.to_string()})
            };
        }

        Ok(self.memory)
    }

    fn translate_instruction(instruction: &str, operands: &str) -> Result<[u8;3], InvaildTokenError>{
        let mut opcode : u8;
        match instruction {
            "NOP" => opcode = 0b00,
            "MOV" => {
                opcode = 0b01000000;
                let left_register = Self::translate_register(operands)?;
                let right_register = Self::translate_register(operands)?;
                opcode &= (left_register << 3) & right_register;
            }
            _ => return Err(InvaildTokenError{ token: instruction.into()})
        }
        Ok([opcode,0,0])
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
            "HL" | "H" => Ok(0b11),
            "SP" | "PSW" => Ok(0b11),
            _ => Err(InvaildTokenError{ token: register_pair.into()})
        }
    }

    fn add_jump_point(&mut self, label: &str) -> Result<(), DuplicateLabelError> {
        if self.jump_map.contains_key(label){
            return Err(DuplicateLabelError{ label: label.into()})
        }

        self.jump_map.insert(label.into(), self.memory_pointer);
        Ok(())
    }
}