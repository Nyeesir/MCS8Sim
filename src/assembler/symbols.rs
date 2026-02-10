use super::{Assembler, DATA_STATEMENTS, INSTRUCTIONS, PSEUDO_INSTRUCTIONS};
use super::errors::{InvalidTokenError, TokenType};
#[derive(Clone)]
pub struct Macro {
    pub params: Vec<String>,
    pub body: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MacroScope {
    pub name: String,
    pub id: u64,
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SymbolScope {
    Global,
    Local(MacroScope),
}

impl Assembler {
    pub fn define_label(&mut self, name: &str) -> Result<(), InvalidTokenError> {
        let name = name.to_uppercase();
        //DOSTAJEMY TERAZ LABEL Z DWUKROPKAMI

        let (name, scope) = if let Some(base) = name.strip_suffix("::") {
            if self.current_macro_scope.is_none() {
                return Err(InvalidTokenError {
                    token: name.into(),
                    token_type: TokenType::Label,
                    additional_info: Some("Labels with :: only work in macros".into()),
                });
            }
            (base, SymbolScope::Global)
        } else if let Some(base) = name.strip_suffix(':') {
            let scope = self
                .current_macro_scope
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

        self.validate_symbol_name_and_check_repeats(&name, SymbolKind::Label, &scope)?;

        let key = self.symbol_key_for_scope(&name, &scope);
        self.symbols.insert(key, Symbol {
            value: self.memory_pointer as i32,
            kind: SymbolKind::Label,
            symbol_scope: scope,
        });

        Ok(())
    }

    pub fn validate_name(&self, name: &str) -> Result<(), InvalidTokenError> {
        if !name.is_ascii() {return Err(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Names can only contain ASCII characters".into())})}

        let first_char = name.chars().next().ok_or(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Name is empty".into())})?;
        if !['@', '?', ':'].contains(&first_char) && !first_char.is_ascii_alphabetic() {return Err(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Names cannot begin with a decimal digit or special character".into())});}

        if INSTRUCTIONS.contains(&name) || PSEUDO_INSTRUCTIONS.contains(&name) || DATA_STATEMENTS.contains(&name){ return Err(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Names cannot be the same as an instruction or a pseudo-instruction".into())});}
        //TODO: ZASTANOWIĆ SIĘ CZY DORZUCIĆ MACRO NAMES

        Ok(())
    }

    pub fn validate_symbol_name_and_check_repeats(&self, name: &str, symbol_kind: SymbolKind, scope: &SymbolScope) -> Result<(), InvalidTokenError>{
        self.validate_name(name)?;

        if self.macros.contains_key(&name.to_uppercase()){
            return Err(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("There already exist a macro with such name".into())});
        }
        let key = self.symbol_key_for_scope(name, scope);
        if let Some(symbol) = self.symbols.get(&key) {
            return if symbol_kind == SymbolKind::Set && symbol.kind == SymbolKind::Set {
                Ok(())
            } else {
                Err(InvalidTokenError { token: name.into(), token_type: TokenType::Label, additional_info: Some("Symbol already defined".into()) })
            }
        }

        Ok(())
    }

    pub fn assert_valid_symbol_name(&self, name: &Option<String>, symbol_kind: SymbolKind, scope: &SymbolScope) -> Result<String, InvalidTokenError>{
        let name = match name {
            Some(name) => name,
            None => return Err(InvalidTokenError { token: "".into(), token_type: TokenType::Label, additional_info: Some("Label is empty".into()) })
        };
        self.validate_symbol_name_and_check_repeats(name.as_str(), symbol_kind, scope)?;
        Ok(name.clone())
    }

    pub fn set_symbol(&mut self, name: String, value: i32, symbol_kind: SymbolKind){
        if let Some(macro_scope) = &self.current_macro_scope {
            match symbol_kind {
                SymbolKind::Set => {
                    let global_key = self.symbol_key_for_scope(&name, &SymbolScope::Global);
                    if let Some(symbol) = self.symbols.get_mut(&global_key) {
                        if symbol.kind == SymbolKind::Set {
                            symbol.value = value;
                            return;
                        }
                    }

                    let local_key = self.symbol_key_for_scope(&name, &SymbolScope::Local(macro_scope.clone()));
                    if let Some(symbol) = self.symbols.get_mut(&local_key) {
                        symbol.value = value;
                    } else {
                        self.symbols.insert(local_key, Symbol{value, kind: SymbolKind::Set, symbol_scope: SymbolScope::Local(macro_scope.clone())});
                    }
                }
                SymbolKind::Equ => {
                    let key = self.symbol_key_for_scope(&name, &SymbolScope::Local(macro_scope.clone()));
                    self.symbols.insert(key, Symbol{value, kind: SymbolKind::Equ, symbol_scope: SymbolScope::Local(macro_scope.clone())});
                }
                //should not be possible, we do nothing
                SymbolKind::Macro => {panic!()}
                SymbolKind::Label => {
                    let scope = SymbolScope::Local(macro_scope.clone());
                    let key = self.symbol_key_for_scope(&name, &scope);
                    self.symbols.insert(key, Symbol{value, kind: SymbolKind::Set, symbol_scope: scope});
                }
            }
        } else {
            let scope = SymbolScope::Global;
            let key = self.symbol_key_for_scope(&name, &scope);
            self.symbols.insert(key, Symbol{value, kind: symbol_kind, symbol_scope: scope });
        }
    }

    pub(crate) fn symbol_key_for_scope(&self, name: &str, scope: &SymbolScope) -> String {
        let name = name.to_uppercase();
        match scope {
            SymbolScope::Global => name,
            SymbolScope::Local(m) => format!("{}@{}", name, m.id),
        }
    }
}
