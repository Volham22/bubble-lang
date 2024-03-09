use crate::ast::{self, Bindable, TypeKind, Visitor};

use super::TypeCheckerError;

pub(crate) struct SoundChecker<'ast> {
    type_name: &'ast str,
}

impl<'ast> SoundChecker<'ast> {
    pub fn new(type_name: &'ast str) -> Self {
        Self { type_name }
    }

    pub fn check(&mut self, stmt: &'ast ast::StructStatement) -> Result<(), TypeCheckerError> {
        for (ty, _) in &stmt.fields {
            self.visit_type(ty)?;
        }

        Ok(())
    }
}

impl<'ast> Visitor<'ast, TypeCheckerError> for SoundChecker<'ast> {
    fn visit_struct(&mut self, stmt: &'ast ast::StructStatement) -> Result<(), TypeCheckerError> {
        if stmt.name == self.type_name {
            return Err(TypeCheckerError::SelfReferentialStruct(
                stmt.name.to_owned(),
            ));
        }

        Ok(())
    }

    fn visit_type(&mut self, ty: &'ast ast::Type) -> Result<(), TypeCheckerError> {
        match &ty.kind {
            TypeKind::Identifier(_) => self.visit_struct(ty.get_struct_def()),
            _ => Ok(()),
        }
    }
}
