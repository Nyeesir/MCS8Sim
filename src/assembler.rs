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

enum OperandType{
    Register,
    RegisterPair,
    Immediate,
    Address,
}

// struct AssemblerData{
//     memory: [u8; MEMORY_SIZE],
//     memory_pointer: usize,
//     jump_map: HashMap<String, usize>
// }

pub fn assemble (data: &str) -> Result<[u8; MEMORY_SIZE], AssemblyError> {
    let _memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
    let _pointer: usize = 0;
    let mut jump_map = HashMap::new();
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
            let token = token.replace(":","");
            jump_map.insert(token, line_number);
            instruction = match(tokens_iter.next()) {
                Some(x) => x,
                None => continue
            };
        }
        else {
            instruction = token;
        }
        let operand = (tokens_iter.next()).unwrap_or_else(|| "");
        let opcodes = match (translate_instruction(instruction, operand)){
            Ok(x) => x,
            Err(e) => return Err(AssemblyError{ line_number, message:e.to_string()})
        };
    }

    Ok(_memory)
}

fn translate_instruction(instruction: &str, operands: &str) -> Result<u8, InvaildTokenError>{
    let opcode : u8;
    match instruction {
        "NOP" => opcode = 0x00,
        _ => return Err(InvaildTokenError{ token: instruction.into()})
    }
    Ok((0x00))
}

fn translate_operand(operands: &str, operand_type: OperandType) -> Result<[u8;2], InvaildTokenError>{
    match operand_type {
        OperandType::Register => {

        }
        OperandType::RegisterPair => {

        }
        OperandType::Immediate => {

        }
        OperandType::Address => {

        }
    }
    Ok([0,0])
}