use std::collections::{HashMap, VecDeque};
use heraclitus_compiler::compiling::{syntax, Metadata, SyntaxResult};
use heraclitus_compiler::prelude::*;
use heraclitus_compiler::{error, syntax_name};
use crate::modules::expression::expr::{Expr, ExprType};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use std::string::String;
use std::sync::{Arc, Mutex, Once};
use itertools::Itertools;
use once_cell::unsync::Lazy;
use crate::docs::module::DocumentationModule;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;

#[derive(Debug, Clone)]
pub struct Dictionary {
    dict: HashMap<String, Expr>,
    kind: Type,
    id: Option<usize>,
}

impl Dictionary {
    pub fn get_name(&self) -> String{
        format!("__AMBER_DICT_{}", self.id.unwrap())
    }

    pub fn gen_id(&mut self, meta: &mut ParserMetadata){
        self.id = Some(meta.gen_var_id());
    }

    pub fn set_id(&mut self, id: usize){
        self.id = Some(id);
    }

    pub fn get_dict(&self) -> HashMap<String, Expr> {
        self.dict.clone()
    }

    pub fn insert(&mut self, key: String, expr: Expr) -> Option<Expr> {
        self.dict.insert(key, expr)
    }

    pub fn dict_from_expr(expr: Expr, meta: &mut TranslateMetadata) -> Option<Dictionary> {
        match expr.value{
            Some(ExprType::Dictionary(dict)) => Some(dict),
            Some(ExprType::VariableGet(var)) => meta.var_to_dict.get(&var.global_id.unwrap()).cloned(),
            _ => None
        }
    }
}

impl Typed for Dictionary {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Dictionary {
    syntax_name!("Dictionary");
    fn new() -> Self {
        Dictionary { dict: HashMap::new(), kind: Type::Dict, id: None }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "{")?;
        context!({
            while let Ok(key) = variable(meta, variable_name_extensions()) {
                token(meta, ":")?;
                let mut expr = Expr::new();
                syntax(meta, &mut expr)?;
                let tok = meta.get_current_token();
                if token(meta, ",").is_ok(){
                    self.dict.insert(key, expr);
                    continue;
                }
                if token(meta, "}").is_ok(){
                    self.gen_id(meta);
                    self.dict.insert(key, expr);
                    return Ok(());
                }
                return error!(meta, tok, "Expected , or }");
            }
            token(meta, "}")?;
            Ok(())
        }, |position| {
            error_pos!(meta, position, "Could not parse dictionary")
        })
    }
}

impl TranslateModule for Dictionary {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = self.get_name();
        let mut keys = vec![];
        let quote = meta.gen_quote();
        for (key, expr) in &self.dict {
            let value = expr.translate_eval(meta, false);
            meta.stmt_queue.push_back(format!("{name}_{key}={value}"));
            keys.push(format!("{name}_{key}"));
        };
        let keys_array = keys.iter().map(|key| format!("{quote}{key}{quote}")).join(" ");
        meta.stmt_queue.push_back(format!("{name}=({keys_array})"));
        let dollar = meta.gen_dollar();
        format!("{quote}{dollar}{{{name}[@]}}{quote}")
    }
}

impl DocumentationModule for Dictionary {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}