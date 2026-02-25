use super::Assembler;
use super::errors::{InvalidTokenError, TokenType};

impl Assembler {
    pub fn parse_register(operand: &str) -> Result<u8, InvalidTokenError>{
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

    pub fn parse_register_pair(operand: &str) -> Result<u8, InvalidTokenError>{
        let register_pair_in_upper = operand.to_uppercase();
        let register_pair = register_pair_in_upper.as_str();
        match register_pair {
            "BC" | "B" => Ok(0b00),
            "DE" | "D" => Ok(0b01),
            "HL" | "H" => Ok(0b10),
            "SP" | "PSW" => Ok(0b11),
            _ => {
                Err(InvalidTokenError { token: register_pair.into(), token_type: TokenType::Operand, additional_info: Some("Only register pairs as words are allowed".into())})
            }
        }
    }

    pub fn parse_16bit_expr(&mut self, expr: &str, offset: usize) -> Result<(u8, u8), InvalidTokenError> {
        match self.calculate_expression(expr, offset, true)? {
            Some(v) => {
                if (-32768..=65535).contains(&v) {
                    let val = v as i16 as u16;
                    Ok(val.to_le_bytes().into())
                } else {
                    Err(InvalidTokenError {
                        token: expr.into(),
                        token_type: TokenType::Operand,
                        additional_info: Some("Expression does not fit in signed 16 bits".into()),
                    })
                }
            }
            None => Ok((0, 0)),
        }
    }

    pub fn parse_positive_16bit_expr_immediately(&mut self, expr: &str) -> Result<u16, InvalidTokenError> {
        match self.calculate_expression(expr, 0, false)? {
            Some(v) if v>= 0 => {
                Ok(v as u16)
            }
            Some(_) => Err(InvalidTokenError { token: expr.into(), token_type: TokenType::Operand, additional_info: Some("Value cannot be nagative".into())}),
            None => Err(InvalidTokenError { token: expr.into(), token_type: TokenType::Operand, additional_info: Some("Expression cannot be immediately evaluated".into())}),
        }
    }

    pub fn parse_8bit_expr(&mut self, expr: &str, offset: usize) -> Result<u8, InvalidTokenError> {
        match self.calculate_expression(expr, offset,true)? {
            Some(v) => {
                if (-128..=255).contains(&v) {
                    Ok(v as i8 as u8)
                } else {
                    Err(InvalidTokenError {
                        token: expr.into(),
                        token_type: TokenType::Operand,
                        additional_info: Some("Expression does not fit in signed 8 bits".into()),
                    })
                }
            }
            None => Ok(0),
        }
    }

    pub fn parse_number_i32(number: &str) -> Result<i32, InvalidTokenError>{
        let value = number.to_uppercase();

        let (number, radix) = match value.chars().last() {
            Some('O') | Some('Q') => (&value[0..value.len()-1], 8),
            Some('B') => (&value[0..value.len()-1], 2),
            //TODO: zastanowic sie czy musi byc numer na poczatku czy nie
            // Some('H') if value.starts_with(&['-','0','1','2','3','4','5','6','7','8','9']) => (&value[0..value.len()-1], 16),
            Some('H') => (&value[0..value.len()-1], 16),
            Some('D') => (&value[0..value.len()-1], 10),
            Some(_) => (value.as_str(), 10),
            None => {Err(InvalidTokenError { token: value.clone(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within valid range with right suffixes are allowed".into())})}?
        };

        i32::from_str_radix(number, radix).map_err(|_| InvalidTokenError { token: value.into(), token_type: TokenType::Operand, additional_info: Some("Only numeric values within valid range with right suffixes are allowed".into())})
    }

    pub fn parse_8bit_number(number: &str) -> Result<u8, InvalidTokenError>{
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

    pub fn assert_operand_amount(operands: &Option<Vec<String>>, allowed_amount: usize) -> Result<&Vec<String>, InvalidTokenError>{
        return match operands {
            Some(operands) => {
                if operands.len() < allowed_amount {
                    Err(InvalidTokenError { token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Too few operands".into()) })
                } else if operands.len() > allowed_amount {
                    Err(InvalidTokenError { token: operands.join(",").into(), token_type: TokenType::Operand, additional_info: Some("Too many operands".into()) })
                } else {
                    Ok(operands)
                }
            }
            None => Err(InvalidTokenError { token: "".into(), token_type: TokenType::Operand, additional_info: Some("Too few operands".into()) })
        }
    }
}