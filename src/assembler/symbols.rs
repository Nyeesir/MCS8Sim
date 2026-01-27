use super::{Assembler, INSTRUCTIONS, PSEUDO_INSTRUCTIONS};
use super::errors::{InvalidTokenError, TokenType};
pub struct Macro {
    pub params: Vec<String>,
    pub body: Vec<String>,
}
pub struct Symbol {
    pub value: i32,
    pub kind: SymbolKind,
    pub symbol_scope: SymbolScope,
}

#[derive(PartialEq)]
pub enum SymbolKind {
    Label,
    Equ,
    Set,
    Macro
}

#[derive(PartialEq)]
pub enum SymbolScope {
    Global,
    Local(String)
}

impl Assembler {
    pub fn define_label(&mut self, name: &str) -> Result<(), InvalidTokenError> {
        let name = name.to_uppercase();
        //DOSTAJEMY TERAZ LABEL Z DWUKROPKAMI

        let (name, scope) = if let Some(base) = name.strip_suffix("::") {
            if self.current_macro_name.is_none() {
                return Err(InvalidTokenError {
                    token: name.into(),
                    token_type: TokenType::Label,
                    additional_info: Some("Labels with :: only work in macros".into()),
                });
            }
            (base, SymbolScope::Global)
        } else if let Some(base) = name.strip_suffix(':') {
            let scope = self
                .current_macro_name
                .as_ref()
                .map(|m| SymbolScope::Local(m.clone()))
                .unwrap_or(SymbolScope::Global);
            (base, scope)
        } else {
            return Err(InvalidTokenError {
                token: name.into(),
                token_type: TokenType::Label,
                additional_info: Some(": should be at the end of the label".into()),
            });
        };

        self.validate_symbol_name_and_check_repeats(&name, SymbolKind::Label)?;

        self.symbols.insert(
            name.to_owned(),
            Symbol {
                value: self.memory_pointer as i32,
                kind: SymbolKind::Label,
                symbol_scope: scope,
            },
        );

        Ok(())
    }

    pub fn validate_name(&self, name: &str) -> Result<(), InvalidTokenError> {
        if !name.is_ascii() {return Err(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Names can only contain ASCII characters".into())})}

        let first_char = name.chars().next().ok_or(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Name is empty".into())})?;
        if !['@', '?', ':'].contains(&first_char) && !first_char.is_ascii_alphabetic() {return Err(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Names cannot begin with a decimal digit or special character".into())});}

        if INSTRUCTIONS.contains(&name) || PSEUDO_INSTRUCTIONS.contains(&name){ return Err(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Names cannot be the same as an instruction or a pseudo-instruction".into())});}

        Ok(())
    }

    pub fn validate_symbol_name_and_check_repeats(&self, name: &str, symbol_kind: SymbolKind) -> Result<(), InvalidTokenError>{
        //TODO: ZASTANOWIC SIE NAD TYM W ODNIESIENIU DO SET SYMBOL
        self.validate_name(name)?;

        if self.macros.contains_key(&name.to_uppercase()){
            return Err(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("There already exist a macro with such name".into())});
        }
        if let Some(symbol) = self.symbols.get(&name.to_uppercase()) {
            return if symbol_kind == SymbolKind::Set && symbol.kind == SymbolKind::Set {
                Ok(())
            } else {
                Err(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Symbol already defined".into()) })
            }
        }

        Ok(())
    }

    pub fn assert_valid_symbol_name(&self, name: &Option<String>, symbol_kind: SymbolKind) -> Result<String, InvalidTokenError>{
        //TODO: ZASTANOWIC SIE NAD TYM W ODNIESIENIU DO SET SYMBOL
        let name = match name {
            Some(name) => name,
            None => return Err(InvalidTokenError { token: "".into(), token_type: TokenType::Label, additional_info: Some("Label is empty".into()) })
        };
        self.validate_symbol_name_and_check_repeats(name.as_str(), symbol_kind)?;
        Ok(name.clone())
    }

    pub fn set_symbol(&mut self, name: String, value: i32, symbol_kind: SymbolKind){
        //TODO POPRAWIONE DEFINE LABEL, CO DALEJ
        if let Some(macro_name) = &self.current_macro_name {
            match symbol_kind {
                SymbolKind::Set => {
                    if let Some(symbol) = self.symbols.get_mut(&name) {
                        symbol.value = value;
                    } else {
                        self.symbols.insert(name.clone(), Symbol{value, kind: SymbolKind::Set, symbol_scope: SymbolScope::Local(macro_name.clone())});
                    }
                }
                SymbolKind::Equ => {
                    self.symbols.insert(name.clone(), Symbol{value, kind: SymbolKind::Equ, symbol_scope: SymbolScope::Local(macro_name.clone())});
                }
                //should not be possible, we do nothing
                SymbolKind::Macro => {panic!()}
                SymbolKind::Label => {
                    let mut scope: SymbolScope;
                    if name.ends_with("::") {
                        scope = SymbolScope::Global
                    } else {
                        scope = SymbolScope::Local(macro_name.clone())
                    }
                    self.symbols.insert(name.clone(), Symbol{value, kind: SymbolKind::Set, symbol_scope: scope});
                }
            }
        } else {
            self.symbols.insert(name.clone(), Symbol{value, kind: symbol_kind, symbol_scope: SymbolScope::Global });
        }
    }
}