use std::convert::Infallible;

use crate::ast::{
    ArrayInitializer, Assignment, BinaryOperation, Call, Expression, Literal, LiteralType,
    MutableVisitor,
};

use super::{Typable, Type};

pub(crate) struct ExpressionTypeSetter<'ty> {
    new_type: &'ty Type,
}

impl<'ty> ExpressionTypeSetter<'ty> {
    pub fn new(new_type: &'ty Type) -> Self {
        Self { new_type }
    }

    pub fn set_type_recusively(&mut self, expr: &mut Expression) {
        self.visit_expression(expr).expect("Should never fail");
    }
}

impl<'ast, 'ty> MutableVisitor<'ast, Infallible> for ExpressionTypeSetter<'ty> {
    fn visit_binary_operation(
        &mut self,
        expr: &'ast mut BinaryOperation,
    ) -> Result<(), Infallible> {
        self.visit_expression(&mut expr.left)?;
        if let Some(right) = &mut expr.right {
            self.visit_expression(right)?;
        }

        expr.set_type(self.new_type.clone());
        Ok(())
    }

    fn visit_literal(&mut self, expr: &'ast mut Literal) -> Result<(), Infallible> {
        match &mut expr.literal_type {
            LiteralType::ArrayAccess(aa) => {
                aa.set_type(self.new_type.clone());
            }
            LiteralType::Null(n) => n.set_type(self.new_type.clone()),
            _ => (),
        }

        expr.set_type(self.new_type.clone());
        Ok(())
    }

    fn visit_assignment(&mut self, expr: &'ast mut Assignment) -> Result<(), Infallible> {
        expr.set_type(self.new_type.clone());
        Ok(())
    }

    fn visit_call(&mut self, expr: &'ast mut Call) -> Result<(), Infallible> {
        expr.set_type(self.new_type.clone());
        Ok(())
    }

    fn visit_array_initializer(
        &mut self,
        expr: &'ast mut ArrayInitializer,
    ) -> Result<(), Infallible> {
        for exp in expr.values.iter_mut() {
            self.visit_expression(exp)?;
        }

        expr.set_type(Type::Array {
            size: expr.values.len() as u32,
            array_type: Box::new(self.new_type.clone()),
        });

        Ok(())
    }
}
