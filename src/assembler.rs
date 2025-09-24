use std::{error::Error, fmt, collections::HashMap};

//TODO: Dodac instrukcje rezerwacji przestrzeni, macro
//TODO: Dodac ewaluacje wyrazen arytmetycznych i logicznych jako operandow (strona 10) i dostosowac do tego parsowanie tokenow

const MEMORY_SIZE: usize = u16::MAX as usize + 1;

const INSTRUCTIONS: [&str; 78] = ["STC", "CMC", "INR", "DCR", "CMA", "DAA", "NOP", "MOV", "STAX", "LDAX"
    , "ADD", "ADC", "SUB", "SBB", "ANA", "XRA", "ORA", "CMP", "RLC", "RRC", "RAL", "RAR", "PUSH"
    , "POP", "DAD", "INX", "DCX", "XCHG", "XTHL", "SPHL", "LXI", "MVI", "ADI", "ACI", "SUI", "SBI", "ANI"
    , "XRI", "ORI", "CPI", "STA", "LDA", "SHLD", "LHLD", "PCHL", "JMP", "JC", "JNC", "JZ", "JNZ", "JP", "JM", "JPE", "JPO"
    , "CALL", "CC", "CNC", "CZ", "CNZ", "CP", "CM", "CPE", "CPO", "RET", "RC", "RNC", "RZ", "RNZ", "RM", "RP", "RPE", "RPO"
    , "RST", "EI", "DI", "IN", "OUT", "HLT"];
const PSEUDO_INSTRUCTIONS: [&str; 8] = ["ORG", "EQU", "SET", "END", "IF", "END IF", "MACRO", "END M"];
const _DATA_STATEMENTS: [&str; 3] = ["DB", "DW", "DS"];

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

            let mut instruction: &str = "";
            let mut label: &str = "";
            let mut operands : Vec<&str> = Vec::new();

            let token = match tokens_iter.next() {
                Some(x) => x,
                None => return Err(AssemblyError{ line_number, line_text: line, message:"Non-empty line doesn't contain a word somehow".into()})
            };

            if token.ends_with(":") {
                label = token;
            } else {
                instruction = token;
            }

            if instruction.is_empty() {
                instruction = tokens_iter.next().unwrap_or_else(|| "");
            }

            for token in tokens_iter {
                if token.starts_with(";") {break; }
                else {operands.push(token); }
            }

            // if Self::handle_pseudo_instruction(self, label, instruction, &operands).is_ok() {continue}
            // if Self::handle_macro().is_ok() {continue}


            if !label.is_empty() {
                match Self::add_jump_point(self, label) {
                    Ok(_) => {},
                    Err(e) => return Err(AssemblyError { line_number, line_text: line, message: e.to_string() })
                }
            }

            if !instruction.is_empty() {
                let binary_values = match Self::translate_instruction(self, instruction, operands) {
                    Ok(x) => x,
                    Err(e) => return Err(AssemblyError { line_number, line_text: line, message: e.to_string() })
                };

                for opcode in binary_values.iter() {
                    self.memory[self.memory_pointer] = opcode.to_owned();
                    self.memory_pointer += 1;
                    if self.memory_pointer >= MEMORY_SIZE {
                        return Err(AssemblyError { line_number, line_text: line, message: "Memory overflow".into() })
                    }
                }
            }
        }

        Ok(self.memory)
    }

    fn translate_instruction(&self, instruction: &str, operands: Vec<&str>) -> Result<Vec<u8>, InvaildTokenError>{
        let instruction_in_upper = instruction.to_uppercase();
        let instruction = instruction_in_upper.as_str();
        //DATA STATEMENTS OMINALEM

        let mut binary_values: Vec<u8> = Vec::with_capacity(3);
        match instruction {
            "STC" => binary_values.push(0b00110111),
            "CMC" => binary_values.push(0b00111111),
            "INR" => {
                binary_values.push(0b00000100);
                let register = Self::parse_register(&operands)?;
                binary_values[0] |= register << 3;
            }
            "DCR" => {
                binary_values.push(0b00000101);
                let register = Self::parse_register(&operands)?;
                binary_values[0] |= register << 3;
            }
            "CMA" => binary_values.push(0b00101111),
            "DAA" => binary_values.push(0b00100111),
            "NOP" => binary_values.push(0b00000000),
            "MOV" => {
                binary_values.push(0b01000000);
                let (left_register, right_register) = Self::parse_2_separate_registers(&operands)?;
                binary_values[0] |= (left_register << 3) & right_register;
            }
            "STAX" | "LDAX" => {
                let register_pair = Self::parse_register_pair(&operands)?;
                match operands[0] {
                    "BC" | "B" | "DE" | "D" => {}
                    _ => return Err(InvaildTokenError{ token: operands[0].into(), token_type: TokenType::Operand, additional_info: Some("Only BC, B, DE, D are allowed".into())})
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
                    let register = Self::parse_register(&operands)?;
                    binary_values[0] |= register;
            }
            "RLC" => binary_values.push(0b00000111),
            "RRC" => binary_values.push(0b00001111),
            "RAL" => binary_values.push(0b00010111),
            "RAR" => binary_values.push(0b00011111),
            "PUSH" => {
                binary_values.push(0b11000101);
                let register_pair = Self::parse_register_pair(&operands)?;
                binary_values[0] |= register_pair<<4;
            }
            //TODO: Mozliwe ze trzeba dodac weryfikacje operandow tzn przyjmowac tylko psw albo sp w zaleznosci od instrukcji itd. Pewnie useless ale moze bedzie trzeba
            "POP" => {
                binary_values.push(0b11000001);
                let register_pair = Self::parse_register_pair(&operands)?;
                binary_values[0] |= register_pair<<4;
            }
            "DAD" => {
                binary_values.push(0b00001001);
                let register_pair = Self::parse_register_pair(&operands)?;
                binary_values[0] |= register_pair<<4;
            }
            "INX" => {
                binary_values.push(0b00000011);
                let register_pair = Self::parse_register_pair(&operands)?;
                binary_values[0] |= register_pair<<4;
            }
            "DCX" => {
                binary_values.push(0b00001011);
                let register_pair = Self::parse_register_pair(&operands)?;
                binary_values[0] |= register_pair<<4;
            }
            "XCHG" => binary_values.push(0b11101011),
            "XTHL" => binary_values.push(0b11100011),
            "SPHL" => binary_values.push(0b11111001),
            "LXI" => {
                binary_values.push(0b00000001);
                let (register_pair, operand) = Self::split_to_2_operands(&operands)?;
                let register_pair = Self::translate_register_pair(&register_pair)?;
                binary_values[0] |= register_pair << 4;
                for value in Self::translate_label_or_address(self, &operand)?{
                    binary_values.push(value);
                }
            }
            "MVI" => {
                binary_values.push(0b00000110);
                let (register, operand) = Self::split_to_2_operands(&operands)?;
                let register = Self::translate_register(&register)?;
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
                    for value in Self::parse_label_or_address(self, &operands)?{
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
                    for value in Self::parse_label_or_address(self, &operands)?{
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
                for value in Self::parse_label_or_address(self, &operands)?{
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
                Self::check_if_right_amount_of_operands(&operands, 1)?;
                match Self::parse_number_u8(operands[0]) {
                    Ok(x) => {
                        if x < 8 {
                            binary_values[0] |= x<<3;
                        } else {
                            return Err(InvaildTokenError{ token: operands[0].into(), token_type: TokenType::Operand, additional_info: Some("RST number is out of range".into())})
                        }
                    }
                    Err(_) => return Err(InvaildTokenError{ token: operands[0].into(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within u8 range are allowed".into())})
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
                Self::check_if_right_amount_of_operands(&operands, 1)?;
                match Self::parse_number_u8(operands[0]) {
                    Ok(x) => binary_values.push(x),
                    Err(_) => return Err(InvaildTokenError{ token: operands[0].into(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within u8 range are allowed".into())})
                }
            }
            "HLT" => binary_values.push(0b01110110),
            _ => return Err(InvaildTokenError{ token: instruction.into(), token_type: TokenType::Instruction, additional_info: None})
        }
        Ok(binary_values)
    }

    fn split_to_2_operands (operands: &Vec<&str>) -> Result<(String, String), InvaildTokenError> {

        if operands.len() == 0 {
            return Err(InvaildTokenError{ token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Operand is missing".into())})
        } else if operands.len() == 1 {
            return match operands[0].split_once(",") {
                Some((first_operand, second_operand)) => { Ok((first_operand.to_owned(), second_operand.to_owned())) },
                None => { Err(InvaildTokenError { token: operands[0].into(), token_type: TokenType::Operand, additional_info: Some("Invalid operand".into()) }) }
            }
        } else if operands.len() == 2 {
            return Ok((operands[0].to_owned().replace(",",""), operands[1].to_owned().replace(",","")))
        }
        let operand_iter = operands.iter();
        let mut first_operand: String = "".to_owned();
        let mut second_operand: String = "".to_owned();

        let mut first_operand_ended = false;
        for token in operand_iter{
            if token == &"," {
                first_operand_ended = true;
                continue
            } else if token.starts_with(",") {
                second_operand += &token.replace(",","");
                first_operand_ended = true;
            } else if token.ends_with(",") {
                first_operand += &token.replace(",","");
                first_operand_ended = true;
            }

            if !first_operand_ended {
                first_operand += token;
            } else {
                second_operand += token;
            }
        }

        println!("Here:\n '{}' '{}'", first_operand, second_operand);
        Ok((first_operand, second_operand))
    }

    fn parse_2_separate_registers(operands: &Vec<&str>) -> Result<(u8, u8), InvaildTokenError>{
        match Self::split_to_2_operands(operands){
            Ok((first_operand, second_operand)) => {
                let register_1 = Self::translate_register(first_operand.as_str())?;
                let register_2 = Self::translate_register(second_operand.as_str())?;
                Ok((register_1, register_2))
            }
            Err(e) => return Err(e)
        }
    }

    fn parse_register(operands: &Vec<&str>) -> Result<u8, InvaildTokenError>{
        if operands.len() == 0 {
            return Err(InvaildTokenError{ token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Operand is missing".into())})
        } else if operands.len() > 1 {
            return Err(InvaildTokenError{ token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Too many operands".into())})
        }
        Self::translate_register(operands[0])
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
            _ => {
                return match Self::parse_number_u8(register){
                    Ok(x) => {
                        if x < 8 {
                            Ok(x)
                        } else { Err(InvaildTokenError{ token: register.into(), token_type: TokenType::Operand, additional_info: Some("Register number is out of range".into())}) }
                    }
                    Err(_) => Err(InvaildTokenError{ token: register.into(), token_type: TokenType::Operand, additional_info: Some("Only registers as words or their numeric presentation is allowed".into())})
                }
            }
        }
    }

    fn parse_register_pair(operands: &Vec<&str>) -> Result<u8, InvaildTokenError>{
        if operands.len() == 0 {
            return Err(InvaildTokenError{ token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Operand is missing".into())})
        } else if operands.len() > 1 {
            return Err(InvaildTokenError{ token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Too many operands".into())})
        }
        Self::translate_register_pair(operands[0])
    }

    fn translate_register_pair(register_pair: &str) -> Result<u8, InvaildTokenError>{
        let register_pair_in_upper = register_pair.to_uppercase();
        let register_pair = register_pair_in_upper.as_str();
        match register_pair {
            "BC" | "B" => Ok(0b00),
            "DE" | "D" => Ok(0b01),
            "HL" | "H" => Ok(0b10),
            "SP" | "PSW" => Ok(0b11),
            _ => {
                return match Self::parse_number_u8(register_pair){
                    Ok(x) => {
                        if x < 4 {
                            Ok(x)
                        } else { Err(InvaildTokenError{ token: register_pair.into(), token_type: TokenType::Operand, additional_info: Some("Register pair number is out of range".into())}) }
                    }
                    Err(_) => Err(InvaildTokenError{ token: register_pair.into(), token_type: TokenType::Operand, additional_info: Some("Only register pairs as words or their numeric presentation is allowed".into())})
                }
            }
        }
    }

    fn parse_label_or_address(&self, operands: &Vec<&str>) -> Result<[u8;2], InvaildTokenError>{
        if operands.len() == 0 {
            return Err(InvaildTokenError{ token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Operand is missing".into())})
        } else if operands.len() > 1 {
            return Err(InvaildTokenError{ token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Too many operands".into())})
        }
        self.translate_label_or_address(operands[0])
    }

    fn translate_label_or_address(&self, label_or_address: &str) -> Result<[u8;2], InvaildTokenError>{
        //TODO: add relative addresses with dolar sign
        //For now, it's case-insensitive
        if label_or_address == "$" {
            let address_bytes = self.memory_pointer.to_le_bytes();
            return Ok([address_bytes[0], address_bytes[1]]);
        }

        if self.jump_map.contains_key(label_or_address){
            let address_bytes = self.jump_map.get(label_or_address).unwrap().to_le_bytes();
            return Ok([address_bytes[0], address_bytes[1]]);
        }

        let address = label_or_address.to_uppercase();
        if let Ok(x) = u16::from_str_radix(&address, 10){
            return Ok(x.to_le_bytes());
        }
        let address_without_suffix = &address[0..address.len()-1];
        if address.ends_with("D"){
            if let Ok(x) = u16::from_str_radix(address_without_suffix, 10){return Ok(x.to_le_bytes())}
        }
        else if address.ends_with("B"){
            if let Ok(x) = u16::from_str_radix(address_without_suffix, 2){return Ok(x.to_le_bytes())}
        }
        else if address.ends_with("O") || address.ends_with("Q"){
            if let Ok(x) = u16::from_str_radix(address_without_suffix, 8){return Ok(x.to_le_bytes())}
        }
        else if address.ends_with("H"){
            if let Ok(x) = u16::from_str_radix(address_without_suffix, 16){return Ok(x.to_le_bytes())}
        }
        Err(InvaildTokenError{ token: address.into(), token_type: TokenType::Address, additional_info: Some("Only numeric values within u16 range with right suffixes or existing labels are allowed".into())})
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

        return match Self::parse_number_u8(value) {
            Ok(x) => Ok(x),
            Err(_) => Err(InvaildTokenError { token: value.into(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within u8 range with right suffixes or ASCII characters in single quotes are allowed".into()) })
        }
    }

    fn parse_number_u8(number: &str) -> Result<u8, InvaildTokenError>{
        let value = number.to_uppercase();
        if let Ok(x) = u8::from_str_radix(&value, 10){return Ok(x)}
        let value_without_suffix = &value[0..value.len()-1];
        if value.ends_with("D"){
            if let Ok(x) = u8::from_str_radix(value_without_suffix, 10){return Ok(x)}
        }
        else if value.ends_with("B"){
            if let Ok(x) = u8::from_str_radix(value_without_suffix, 2){return Ok(x)}
        }
        else if value.ends_with("O") || value.ends_with("Q"){
            if let Ok(x) = u8::from_str_radix(value_without_suffix, 8){return Ok(x)}
        }
        else if value.ends_with("H") && value.starts_with(&['0','1','2','3','4','5','6','7','8','9']){
            if let Ok(x) = u8::from_str_radix(value_without_suffix, 16){return Ok(x)}
        }
        Err(InvaildTokenError{ token: value.into(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within u8 range with right suffixes are allowed".into())})
    }

    fn add_jump_point(&mut self, label: &str) -> Result<(), InvaildTokenError> {
        let mut label = label.trim();
        label = &label[0..label.len()-1];

        match self.validate_label(label) {
            Ok(()) => {},
            Err(e) => return Err(e)
        }

        if self.jump_map.contains_key(label){
            return Err(InvaildTokenError{token: label.into(), token_type: TokenType::Label, additional_info: Some("Label already exists".into())})
        }

        self.jump_map.insert(label.into(), self.memory_pointer);
        Ok(())
    }

    fn validate_label(&self, label: &str) -> Result<(), InvaildTokenError>{
        //We should allow labels with max 5 chars, but we will skip it for now
        let label_to_upper = label.to_uppercase();
        let label = label_to_upper.as_str();

        if !label.is_ascii() {return Err(InvaildTokenError{ token: label.into(), token_type: TokenType::Label, additional_info: Some("Labels can only contain ASCII characters".into())})}

        let first_char = label.chars().next().ok_or(InvaildTokenError{ token: label.into(), token_type: TokenType::Label, additional_info: Some("Label is empty".into())})?;
        if !(['@', '?', ':'].contains(&first_char) || first_char.is_ascii_alphabetic()) {return Err(InvaildTokenError{ token: label.into(), token_type: TokenType::Label, additional_info: Some("Labels cannot begin with a decimal digit".into())});}

        if INSTRUCTIONS.contains(&label) || PSEUDO_INSTRUCTIONS.contains(&label){ return Err(InvaildTokenError{ token: label.into(), token_type: TokenType::Label, additional_info: Some("Labels cannot be the same as an instruction or a pseudo-instruction".into())});}

        Ok(())
    }

    fn check_if_right_amount_of_operands(operands: &Vec<&str>, allowed_amount: usize) -> Result<(), InvaildTokenError>{
        if operands.len() < allowed_amount{
            return Err(InvaildTokenError{ token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Too less operands".into())})
        } else if operands.len() > allowed_amount{
            return Err(InvaildTokenError{ token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Too many operands".into())})
        }
        Ok(())
    }

    fn handle_pseudo_instruction(&mut self, label: &str, instruction: &str, operands: &Vec<&str>) -> Result<(), InvaildTokenError>{
        match instruction {
            "COSTAM" => unimplemented!(),
            _ => return Err( InvaildTokenError {token: instruction.into(), token_type:TokenType::Instruction, additional_info: Some("It is not a valid pseudo-instruction".into())})
        }
    }

    fn handle_macro() -> Result<(), InvaildTokenError>{
        unimplemented!()
    }
}