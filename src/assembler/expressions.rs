use std::iter::Peekable;
use std::slice::Iter;
use regex::Regex;
use crate::assembler::symbols::SymbolScope;
use super::{symbols, Assembler};
use super::errors::{InvalidTokenAtLineError, InvalidTokenError, TokenType};

#[derive(Debug, Clone)]
enum CalculationToken {
    Num(i32),
    Op(Op),
    LParen,
    RParen,
    Symbol(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Not,
}

#[derive(Debug, Clone)]
enum Expr {
    Value(i32),
    Symbol(String),
    Unary { op: Op, expr: Box<Expr> },
    Binary { op: Op, left: Box<Expr>, right: Box<Expr> },
}

pub struct PendingExpr {
    addr: usize,
    expr: Expr,
    line: usize,
    macro_name: Option<String>,
}

impl Assembler {
    pub fn calculate_expression(
        &mut self,
        expr: &str,
        offset: usize,
        allow_forward_references: bool,
    ) -> Result<Option<i32>, InvalidTokenError> {
        let tokens = Self::tokenize(self, expr)?;
        let mut it = tokens.iter().peekable();

        let ast = Self::parse_expr(&mut it, 0)?;

        if it.peek().is_some() {
            return Err(InvalidTokenError {
                token: expr.into(),
                token_type: TokenType::Operand,
                additional_info: Some("Unexpected token at end of expression".into()),
            });
        }

        match self.eval_expr(&ast, self.current_macro_name.as_deref()) {
            Ok(v) => Ok(Some(v)),
            Err(_) => {
                if allow_forward_references {
                    self.pending_exprs.push(PendingExpr {
                        addr: self.memory_pointer + offset,
                        expr: ast,
                        line: self.current_line,
                        macro_name: self.current_macro_name.clone()
                    });
                } else {
                    return Err(InvalidTokenError {
                        token: expr.into(),
                        token_type: TokenType::Label,
                        additional_info: Some("Forward reference is not allowed".into()),
                    });
                }
                Ok(None)
            }
        }
    }

    fn tokenize(&self, expr: &str) -> Result<Vec<CalculationToken>, InvalidTokenError> {
        let pattern = r"(\bHERE\b|\$|\bMOD\b|\bNOT\b|\bAND\b|\bOR\b|\bXOR\b|\bSHL\b|\bSHR\b|\+|-|\*|/|\(|\))";
        let re = Regex::new(pattern).unwrap();

        let mut tokens = Vec::new();
        let mut last = 0;

        for m in re.find_iter(expr) {
            if m.start() > last {
                Self::push_part_as_token(&expr[last..m.start()], &mut tokens);
            }

            let t = m.as_str();
            tokens.push(match t {
                "(" => CalculationToken::LParen,
                ")" => CalculationToken::RParen,
                "HERE" | "$" => CalculationToken::Num(self.memory_pointer as i32),
                _ => CalculationToken::Op(match t {
                    "+" => Op::Add,
                    "-" => Op::Sub,
                    "*" => Op::Mul,
                    "/" => Op::Div,
                    "MOD" => Op::Mod,
                    "AND" => Op::And,
                    "OR"  => Op::Or,
                    "XOR" => Op::Xor,
                    "SHL" => Op::Shl,
                    "SHR" => Op::Shr,
                    "NOT" => Op::Not,
                    _ => unreachable!(),
                }),
            });

            last = m.end();
        }

        if last < expr.len() {
            Self::push_part_as_token(&expr[last..], &mut tokens);
        }

        Ok(tokens)
    }

    fn push_part_as_token(
        part: &str,
        tokens: &mut Vec<CalculationToken>,
    ) {
        let part = part.trim();
        if part.is_empty() {
            return;
        }

        if part.starts_with('\'') && part.ends_with('\'') && part.len() == 2 {
            tokens.push(CalculationToken::Num(0i32));
        }
        else if part.starts_with('\'') && part.ends_with('\'') && part.len() == 3 {
            let c = part.chars().nth(1).unwrap();
            tokens.push(CalculationToken::Num(c as i32));
        } else if let Ok(v) = Self::parse_number_i32(part) {
            tokens.push(CalculationToken::Num(v));
        } else {
            tokens.push(CalculationToken::Symbol(part.to_string()));
        }
    }

    fn precedence(op: Op) -> u8 {
        match op {
            Op::Or | Op::Xor        => 1,
            Op::And                => 2,
            Op::Not                => 3, // unary
            Op::Add | Op::Sub      => 4,
            Op::Mul | Op::Div | Op::Mod | Op::Shl | Op::Shr => 5,
        }
    }


    fn parse_expr(
        tokens: &mut Peekable<Iter<CalculationToken>>,
        min_prec: u8,
    ) -> Result<Expr, InvalidTokenError> {

        let mut lhs = match tokens.next() {
            Some(CalculationToken::Num(v)) => Expr::Value(*v),

            Some(CalculationToken::Symbol(l)) => Expr::Symbol(l.clone()),

            Some(CalculationToken::Op(op @ Op::Sub)) | Some(CalculationToken::Op(op @ Op::Not)) => {
                let prec = Self::precedence(*op);
                let expr = Self::parse_expr(tokens, prec)?;
                Expr::Unary {
                    op: *op,
                    expr: Box::new(expr),
                }
            }

            Some(CalculationToken::LParen) => {
                let e = Self::parse_expr(tokens, 0)?;
                match tokens.next() {
                    Some(CalculationToken::RParen) => e,
                    _ => return Err(InvalidTokenError {
                        token: "".into(),
                        token_type: TokenType::Operand,
                        additional_info: Some("Missing ')'".into()),
                    }),
                }
            }

            _ => return Err(InvalidTokenError {
                token: "".into(),
                token_type: TokenType::Operand,
                additional_info: Some("Expected expression".into()),
            }),
        };

        while let Some(CalculationToken::Op(op)) = tokens.peek() {
            let prec = Self::precedence(*op);
            if prec < min_prec {
                break;
            }

            let op = *op;
            tokens.next();

            let rhs = Self::parse_expr(tokens, prec + 1)?;
            lhs = Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn eval_bin(op: Op, a: i32, b: i32) -> i32 {
        let r = match op {
            Op::Add => a.wrapping_add(b),
            Op::Sub => a.wrapping_sub(b),
            Op::Mul => a.wrapping_mul(b),
            Op::Div => a / b,
            Op::Mod => a % b,
            Op::And => a & b,
            Op::Or  => a | b,
            Op::Xor => a ^ b,
            Op::Shl => a.wrapping_shl((b & 0xF) as u32),
            Op::Shr => a.wrapping_shr((b & 0xF) as u32),
            _ => unreachable!(), // NOT is unary
        };
        r & 0xFFFF
    }


    fn eval_expr(&self, expr: &Expr, macro_name: Option<&str>) -> Result<i32, String> {
        match expr {
            Expr::Value(v) => Ok(*v),

            Expr::Symbol(l) => {
                let symbol = self
                    .symbols
                    .get(&l.to_uppercase())
                    .ok_or_else(|| format!("Undefined symbol {}", l))?;

                match (&symbol.symbol_scope, macro_name) {
                    // same macro names
                    (SymbolScope::Local(name), Some(macro_name)) if *name == macro_name =>
                        {
                            Ok(symbol.value)
                        }

                    // different macro names
                    (SymbolScope::Local(_), Some(_)) => {
                        Err(format!(
                            "Symbol {} is local to a different macro",
                            l
                        ))
                    }

                    // global
                    (SymbolScope::Global, _) => {
                        Ok(symbol.value)
                    }

                    // macro variable but not in macro
                    (SymbolScope::Local(_), None) => {
                        Err(format!(
                            "Symbol {} is local and cannot be used outside macro",
                            l
                        ))
                    }
                }
            }

            Expr::Unary { op, expr } => {
                let v = self.eval_expr(expr, macro_name)?;
                Ok(match op {
                    Op::Not => !v & 0xFFFF,
                    Op::Sub => (!v).wrapping_add(1), // unary minus
                    _ => unreachable!(),
                })
            }

            Expr::Binary { op, left, right } => {
                let l = self.eval_expr(left, macro_name)?;
                let r = self.eval_expr(right, macro_name)?;
                Ok(Self::eval_bin(*op, l, r))
            }
        }
    }

    pub fn resolve_pending_exprs(&mut self) -> Result<(), InvalidTokenAtLineError> {
        for p in &self.pending_exprs {
            let v = self.eval_expr(&p.expr,  p.macro_name.as_deref())
                .map_err(|e| InvalidTokenAtLineError {
                    line: p.line,
                    source: InvalidTokenError {
                        token: e,
                        token_type: TokenType::Label,
                        additional_info: None,
                    },
                })?;

            let b = v.to_le_bytes();
            self.memory[p.addr] = b[0];
            self.memory[p.addr + 1] = b[1];
        }
        Ok(())
    }
}