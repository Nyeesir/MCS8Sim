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

        let mut tokens_iter = line.trim().split_whitespace();

        let token_option = tokens_iter.next();
        let token = match token_option {
            Some(x) => x,
            None => return Err(AssemblyError{ line_number, message:"Non-empty line doesn't contain a word somehow".into()})
        };

        if token.contains(":") {
            handle_label(token);
            let token = tokens_iter.next();
            match token {
                Some(token) => translate_instruction(token),
                None => (continue)
            }
        }
        else {
            translate_instruction(token)
        }
    }

    Ok(_memory)
}

fn translate_instruction(instruction: &str){
    let opcode : u8;
    let operands : u8;
    match instruction {
        "NOP" => {
            opcode = 0x00;
            operands = 0;
        }
        "ADI" => {

        }
        _ => println!("unknown command")
    }
}

fn handle_label(label: &str){

}