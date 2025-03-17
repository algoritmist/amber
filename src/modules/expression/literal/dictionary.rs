use std::collections::{HashMap, VecDeque};
use heraclitus_compiler::compiling::{syntax, Metadata, SyntaxResult};
use heraclitus_compiler::prelude::*;
use heraclitus_compiler::{error, syntax_name};
use crate::modules::expression::expr::{Expr, ExprType};
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};
use std::string::String;
use itertools::Itertools;
use crate::docs::module::DocumentationModule;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;

#[derive(Debug, Clone)]
pub struct Dictionary {
    dict: HashMap<String, Expr>,
    kind: Type
}

impl Typed for Dictionary {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for Dictionary {
    syntax_name!("Dictionary");
    fn new() -> Self {
        Dictionary { dict: HashMap::new(), kind: Type::default() }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "|")?;
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
                if token(meta, "|").is_ok(){
                    self.dict.insert(key, expr);
                    return Ok(());
                }
                return error!(meta, tok, "Expected , or |");
            }
            token(meta, "|")?;
            Ok(())
        }, |position| {
            error_pos!(meta, position, "Could not parse dictionary")
        })
    }
}

impl TranslateModule for Dictionary {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let name = format!("__AMBER_DICT_{}", meta.gen_value_id());
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