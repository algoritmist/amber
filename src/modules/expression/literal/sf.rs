use std::env::var;
use std::ptr::null;
use heraclitus_compiler::prelude::*;
use heraclitus_compiler::compiling::{token, SyntaxResult, Metadata};
use heraclitus_compiler::{error, syntax_name};
use crate::docs::module::DocumentationModule;
use crate::modules::expression::expr::Expr;
use crate::modules::expression::expr::ExprType::Text;
use crate::modules::types::{Type, Typed};
use crate::modules::variable::variable_name_extensions;
use crate::translate::module::TranslateModule;
use crate::utils::{ParserMetadata, TranslateMetadata};

use crate::modules::expression::binop::BinOp;

#[derive(Debug, Clone)]
pub struct SetField{
    left: String,
    right: Box<Expr>,
    kind: Type
}

impl Typed for SetField{
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl SyntaxModule<ParserMetadata> for SetField{
    syntax_name!("SetField");
    fn new() -> Self {
        SetField{
            left: String::new(),
            right: Box::new(Expr::new()),
            kind: Type::default()
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        //handle_binop!(meta, "add", self.left, self.right, [Num])?;
        /*match self.left.value{
            Some(Text {..}) => Ok(()),
            _ => error!(meta, meta.get_current_token(), "Expected key to be a variable name")
        }*/
        let tok = meta.get_current_token();
        self.left = variable(meta, variable_name_extensions())?;
        context!({
            token(meta, ":")?;
            syntax(meta, &mut *self.right)?;
            Ok(())
        }, |position| {
            error_pos!(meta, position, format!("Expected ':' after variable name '{}'", self.left))
            }
        )
    }
}

impl TranslateModule for SetField{
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        // TODO: fix qoutes in key. Parsing key as variable doesn't work, parsing as Text logically fails. Maybe we need a Field expression?
        let name = format!("__AMBER_DICT_{}_{}", meta.value_id, self.left);
        let value = self.right.translate_eval(meta, false);
        let quote = meta.gen_quote();
        meta.stmt_queue.push_back(format!("{name}=({value})"));
        format!("{quote}{name}{quote}")
    }
}

impl DocumentationModule for SetField {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}