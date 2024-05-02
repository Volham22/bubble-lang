use crate::ast::{self, Bindable, Locatable, TypeKind, Visitor};

use super::TypeCheckerError;

pub(crate) struct SoundChecker<'ast> {
    type_name: &'ast str,
    self_reference: Vec<String>,
}

impl<'ast> SoundChecker<'ast> {
    pub fn new(type_name: &'ast str) -> Self {
        Self {
            type_name,
            self_reference: Vec::new(),
        }
    }

    pub fn check(&mut self, stmt: &'ast ast::StructStatement) -> Result<(), TypeCheckerError> {
        for (ty, _) in &stmt.fields {
            self.visit_type(ty)?;
        }

        Ok(())
    }

    /// Return a slice of strings, of the safe (meaning it's reference using a pointer) self
    /// referential fields.
    pub fn get_safe_self_references(
        &mut self,
        stmt: &'ast ast::StructStatement,
    ) -> Result<&[String], TypeCheckerError> {
        self.check(stmt)?;
        Ok(&self.self_reference)
    }
}

impl<'ast> Visitor<'ast, TypeCheckerError> for SoundChecker<'ast> {
    fn visit_struct(&mut self, stmt: &'ast ast::StructStatement) -> Result<(), TypeCheckerError> {
        if stmt.name == self.type_name {
            return Err(TypeCheckerError::SelfReferentialStruct(
                stmt.get_location().clone(),
                stmt.name.to_owned(),
            ));
        }

        Ok(())
    }

    fn visit_type(&mut self, ty: &'ast ast::Type) -> Result<(), TypeCheckerError> {
        match &ty.kind {
            TypeKind::Identifier(_) => self.visit_struct(ty.get_struct_def()),
            TypeKind::Ptr(inner_ty) => match self.visit_type(inner_ty) {
                // Error recovery here. Self reference through pointer is legal, but we need to
                // be able to detect it to handle it properly
                Err(TypeCheckerError::SelfReferentialStruct(_, name)) => {
                    self.self_reference.push(name);
                    Ok(())
                }
                Err(e) => Err(e),
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }
}
