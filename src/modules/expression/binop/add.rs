use heraclitus_compiler::prelude::*;
use crate::docs::module::DocumentationModule;
use crate::{handle_binop, error_type_match};
use crate::modules::expression::expr::{Expr, ExprType};
use crate::modules::expression::literal::dictionary::{Dictionary};
use crate::translate::compute::{translate_computation, ArithOp};
use crate::utils::{ParserMetadata, TranslateMetadata};
use crate::translate::module::TranslateModule;
use crate::modules::types::{Typed, Type};
use crate::modules::types::Type::Dict;
use super::BinOp;

#[derive(Debug, Clone)]
pub struct Add {
    left: Box<Expr>,
    right: Box<Expr>,
    kind: Type,
    id: Option<usize>
}

impl Typed for Add {
    fn get_type(&self) -> Type {
        self.kind.clone()
    }
}

impl BinOp for Add {
    fn set_left(&mut self, left: Expr) {
        self.left = Box::new(left);
    }

    fn set_right(&mut self, right: Expr) {
        self.right = Box::new(right);
    }

    fn parse_operator(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        token(meta, "+")?;
        Ok(())
    }
}

impl SyntaxModule<ParserMetadata> for Add {
    syntax_name!("Add");

    fn new() -> Self {
        Add {
            left: Box::new(Expr::new()),
            right: Box::new(Expr::new()),
            kind: Type::default(),
            id: None,
        }
    }

    fn parse(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
        self.kind = handle_binop!(meta, "add", self.left, self.right, [
            Num,
            Text,
            Array,
            Dict
        ])?;
        self.id = Some(meta.gen_var_id());
        Ok(())
    }
}

impl TranslateModule for Add {
    fn translate(&self, meta: &mut TranslateMetadata) -> String {
        let left = self.left.translate_eval(meta, false);
        let right = self.right.translate_eval(meta, false);
        match self.kind {
            Type::Array(_) => {
                let quote = meta.gen_quote();
                let dollar = meta.gen_dollar();
                let id = meta.gen_value_id();
                let name = format!("__AMBER_ARRAY_ADD_{id}");
                meta.stmt_queue.push_back(format!("{name}=({left} {right})"));
                format!("{quote}{dollar}{{{name}[@]}}{quote}")
            },
            Type::Dict => {
                let left = Dictionary::dict_from_expr(*self.left.clone(), meta);
                let right = Dictionary::dict_from_expr(*self.right.clone(), meta);
                if let (Some(left_dict), Some(right_dict)) = (left, right) {
                    println!("Successful decomposition");
                    let left_field = Dictionary::field_from_expr(*self.left.clone());
                    let right_field = Dictionary::field_from_expr(*self.right.clone());
                    if let (Some(left_field), Some(right_field)) = (left_field, right_field) {
                        let left_value = &left_dict.get_dict()[&left_field];
                        let right_value = &right_dict.get_dict()[&right_field];
                        let op = ExprType::Add(
                            Add{
                                kind: left_value.kind.clone(),
                                left: Box::new(left_value.clone()),
                                right: Box::new(right_value.clone()),
                                id: None
                            }
                        );
                        let mut expr = Expr::new();
                        expr.value = Some(op);
                        expr.kind = left_value.kind.clone();
                        return expr.translate(meta);
                    }
                    let mut dict = Dictionary::new();
                    for (left_key, left_value) in left_dict.get_dict() {
                        let operation = if let Some(right_value) = right_dict.get_dict().get(&left_key) {
                            let op = ExprType::Add(
                                Add
                                {
                                    kind: left_value.kind.clone(),
                                    left: Box::new(left_value),
                                    right: Box::new(right_value.clone()),
                                    id: None
                                }
                            );
                            let mut expr = Expr::new();
                            expr.value = Some(op);
                            expr.kind = Dict;
                            expr
                        } else {
                            left_value
                        };
                        dict.insert(left_key.clone(), operation);
                    }
                    for (right_key, right_value) in right_dict.get_dict() {
                        if let Some(_) = left_dict.get_dict().get(&right_key) {
                            continue;
                        }
                        dict.insert(right_key.clone(), right_value);
                    }
                    dict.set_id(self.id.unwrap());
                    return dict.translate(meta);
                } else {
                    println!("UnSuccessful decomposition");
                }
                String::new()
            },
            Type::Text => format!("{}{}", left, right),
            _ => translate_computation(meta, ArithOp::Add, Some(left), Some(right))
        }
    }
}

impl DocumentationModule for Add {
    fn document(&self, _meta: &ParserMetadata) -> String {
        "".to_string()
    }
}
