use std::{error::Error, fmt};

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

pub fn assemble (data: &str) -> Result<[u8; MEMORY_SIZE], AssemblyError> {
    let _memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
    let _pointer: usize = 0;
    let mut line_number: usize = 0;

    let lines = data.lines();
    for line in lines{
        line_number += 1;
        if line.is_empty() {continue}

        let line = line.to_uppercase();
        let mut tokens_iter = line.trim().split_whitespace();

        let token = match (tokens_iter.next()) {
            Some(x) => x,
            None => return Err(AssemblyError{ line_number, message:"Non-empty line doesn't contain a word somehow".into()})
        };

        let instruction: &str;
        if token.contains(":") {
            handle_label(token);
            instruction = match(tokens_iter.next()) {
                Some(x) => x,
                None => continue
            };
        }
        else {
            instruction = token;
        }
        let operand = (tokens_iter.next()).unwrap_or_else(|| "");
        translate_instruction(instruction, operand);
    }

    Ok(_memory)
}

fn translate_instruction(instruction: &str, operand: &str){
    let opcode : u8;
    match instruction {
        "NOP" => opcode = 0x00,
        "MOV" => match operand {
            "B,B" => opcode = 0x00,
            _ => opcode = 0x00
        }
        _ => opcode = 0x00
    }

}

fn handle_label(label: &str){

}