use std::ops::Deref;

use crate::ast::{
    self, AddrOf, ArrayInitializer, Assignment, BinaryOperation, Bindable, Call, Definition,
    Expression, ForStatement, FunctionStatement, GlobalStatement, IfStatement, LetStatement,
    Literal, LiteralType, Locatable, MutableVisitor, OpType, ReturnStatement, StructStatement,
    WhileStatement,
};

use super::{
    errors::TypeCheckerError, inference::IntegerInference, type_setter::ExpressionTypeSetter,
    Typable, Type,
};

pub fn run_type_checker(stmts: &mut [GlobalStatement]) -> Result<(), TypeCheckerError> {
    let mut type_checker = TypeChecker::default();
    let mut int_inference = IntegerInference::default();

    type_checker.check_statements(stmts)?;
    int_inference.infer_statements(stmts)?;

    // type_checker.check_statements(stmts)
    Ok(())
}

#[derive(Default)]
pub struct TypeChecker {
    current_type: Option<Type>,
    current_function: Option<Type>, // current's function type
}

impl<'ast> TypeChecker {
    pub fn check_statements(
        &mut self,
        stmts: &'ast mut [GlobalStatement],
    ) -> Result<(), TypeCheckerError> {
        for stmt in stmts.iter_mut() {
            self.visit_global_statement(stmt)?;
            self.current_type = None;
            self.current_function = None;
        }

        Ok(())
    }

    fn check_bool_expression(
        &mut self,
        expr: &'ast mut Expression,
    ) -> Result<(), TypeCheckerError> {
        self.visit_expression(expr)?;

        match self
            .current_type
            .as_ref()
            .expect("expression should have type")
        {
            Type::Bool => Ok(()),
            _ => Err(TypeCheckerError::NonBoolCondition(
                self.current_type.clone().unwrap(),
            )),
        }
    }
}

impl<'ast> MutableVisitor<'ast, TypeCheckerError> for TypeChecker {
    fn visit_function(
        &mut self,
        stmt: &'ast mut FunctionStatement,
    ) -> Result<(), TypeCheckerError> {
        // Set parameters type
        for parameter in stmt.parameters.iter_mut() {
            parameter.set_type(Type::from(
                parameter
                    .declaration_type
                    .clone()
                    .expect("Parameter has no type hint!"),
            ))
        }

        let function_type = Type::Function {
            parameters: stmt
                .parameters
                .iter()
                .map(|let_stmt| (let_stmt.get_type().clone(), let_stmt.name.clone()))
                .collect(),
            return_type: Box::new(stmt.return_type.clone().into()),
        };

        self.current_function = Some(function_type.clone());
        stmt.set_type(function_type);

        if let Some(body) = stmt.body.as_mut() {
            self.visit_statements(body)?;
        }

        self.current_function = None;
        Ok(())
    }

    fn visit_return(&mut self, stmt: &'ast mut ReturnStatement) -> Result<(), TypeCheckerError> {
        if let Some(ref mut exp) = stmt.exp {
            self.visit_expression(exp)?;
        } else {
            self.current_type = Some(Type::Void);
        }

        match self
            .current_function
            .as_ref()
            .expect("return outside a function")
        {
            Type::Function { return_type, .. } => {
                if !return_type.is_compatible_with(
                    self.current_type
                        .as_ref()
                        .expect("return expression has no type"),
                ) {
                    Err(TypeCheckerError::ReturnTypeMismatch {
                        got: self.current_type.clone().unwrap(),
                        expected: return_type.deref().clone(),
                    })
                } else {
                    Ok(())
                }
            }
            _ => unreachable!("current function type is not a function!"),
        }
    }

    fn visit_struct(&mut self, stmt: &'ast mut StructStatement) -> Result<(), TypeCheckerError> {
        stmt.set_type(Type::Struct {
            name: stmt.name.clone(),
            fields: stmt
                .fields
                .iter()
                .map(|(kind, name)| (Type::from(kind.clone()), name.clone()))
                .collect(),
        });

        Ok(())
    }

    fn visit_let(&mut self, stmt: &'ast mut LetStatement) -> Result<(), TypeCheckerError> {
        self.visit_expression(
            stmt.init_exp
                .as_mut()
                .expect("Let statement has no init exp!"),
        )?;

        match &stmt.declaration_type {
            Some(ty) => {
                let real_type: Type = ty.clone().into();

                if !real_type
                    .is_compatible_with(self.current_type.as_ref().expect("let init has no type"))
                {
                    return Err(TypeCheckerError::BadInit {
                        left: real_type,
                        right: self.current_type.clone().unwrap(),
                    });
                }

                // If init expression is null we need to give it its real type. The null type will
                // now hold the concrete type. This is required for the translation pass
                if let Type::Null { .. } = self.current_type.as_ref().expect("Should have a type") {
                    let set_ty = Type::Null {
                        concrete_type: Some(Box::new(real_type.clone())),
                    };
                    let mut setter = ExpressionTypeSetter::new(&set_ty);

                    setter.set_type_recusively(
                        stmt.init_exp
                            .as_mut()
                            .expect("Should have an init exp (is the parser correct?)"),
                    )
                }

                self.current_type = Some(real_type.clone());
                stmt.set_type(real_type);
            }
            None => {
                if let Type::Null { .. } = self.current_type.as_ref().expect("Should have a type") {
                    return Err(TypeCheckerError::InferenceError(
                        stmt.get_location().clone(),
                    ));
                }

                stmt.set_type(self.current_type.as_ref().unwrap().clone());
            }
        }

        Ok(())
    }

    fn visit_if(&mut self, stmt: &'ast mut IfStatement) -> Result<(), TypeCheckerError> {
        self.check_bool_expression(&mut stmt.condition)?;
        self.visit_statements(&mut stmt.then_clause)?;

        if let Some(stmts) = &mut stmt.else_clause {
            self.visit_statements(stmts)?;
        }

        Ok(())
    }

    fn visit_while(&mut self, stmt: &'ast mut WhileStatement) -> Result<(), TypeCheckerError> {
        self.check_bool_expression(&mut stmt.condition)?;
        self.visit_statements(&mut stmt.body)?;

        Ok(())
    }

    fn visit_for(&mut self, stmt: &'ast mut ForStatement) -> Result<(), TypeCheckerError> {
        self.visit_let(&mut stmt.init_decl)?;
        self.check_bool_expression(&mut stmt.continue_expression)?;
        self.visit_expression(&mut stmt.modify_expression)?;
        self.visit_statements(&mut stmt.body)?;

        Ok(())
    }

    fn visit_assignment(&mut self, expr: &'ast mut Assignment) -> Result<(), TypeCheckerError> {
        self.visit_expression(&mut expr.left)?;

        let lhs_ty = self
            .current_type
            .clone()
            .expect("left expression should have a type");

        self.visit_expression(&mut expr.right)?;

        let rhs_ty = self
            .current_type
            .as_ref()
            .expect("left expression should have a type");

        if !lhs_ty.is_compatible_with(rhs_ty) {
            return Err(TypeCheckerError::BadAssigment {
                left: lhs_ty,
                right: rhs_ty.clone(),
            });
        }

        self.current_type = Some(lhs_ty);

        Ok(())
    }

    fn visit_call(&mut self, expr: &'ast mut Call) -> Result<(), TypeCheckerError> {
        if expr.get_definition().is_function() {
            if expr.arguments.len() != expr.get_function_def().parameters.len() {
                return Err(TypeCheckerError::BadParameterCount {
                    expected: expr.get_function_def().parameters.len() as u32,
                    got: expr.arguments.len() as u32,
                });
            }

            // Add parameters types to a vector
            let mut parameter_types = Vec::with_capacity(expr.get_function_def().parameters.len());
            for param_expr in expr.arguments.iter_mut() {
                self.visit_expression(param_expr)?;
                parameter_types.push(
                    self.current_type
                        .clone()
                        .expect("Parameter expression should be typed"),
                );
            }

            for (expr_type, function_parameter) in parameter_types
                .iter()
                .zip(expr.get_function_def().parameters.iter())
            {
                let expected_type = function_parameter
                    .ty
                    .clone()
                    .expect("Parameter should be typed");

                if !expr_type.is_compatible_with(&expected_type) {
                    return Err(TypeCheckerError::BadParameter {
                        name: function_parameter.name.clone(),
                        expected_type,
                        got: self.current_type.clone().unwrap(),
                    });
                }
            }

            // A call expression type is the function return type
            self.current_type = Some(expr.get_function_def().return_type.clone().into());
            Ok(())
        } else {
            Err(TypeCheckerError::NotCallable(expr.get_definition().clone()))
        }
    }

    fn visit_type(&mut self, ty: &'ast mut crate::ast::Type) -> Result<(), TypeCheckerError> {
        self.current_type = Some(ty.kind.clone().into());
        Ok(())
    }

    fn visit_binary_operation(
        &mut self,
        expr: &'ast mut BinaryOperation,
    ) -> Result<(), TypeCheckerError> {
        match expr.right {
            // Binary operation
            Some(ref mut right_exp) => {
                self.visit_expression(&mut expr.left)?;
                let left_ty = self
                    .current_type
                    .clone()
                    .expect("No Left type in binary operation!");

                self.visit_expression(right_exp)?;

                let right_ty = self
                    .current_type
                    .as_ref()
                    .expect("No right type in binary operation!");

                if !left_ty.is_compatible_with(right_ty) {
                    return Err(TypeCheckerError::IncompatibleOperationType {
                        operator: expr.op,
                        left_ty,
                        right_ty: self.current_type.clone().unwrap(),
                    });
                }

                // Plus, Minus, Multiply, Divide and modulo expression has a result of their type
                if matches!(
                    expr.op,
                    OpType::Plus
                        | OpType::Minus
                        | OpType::Multiply
                        | OpType::Divide
                        | OpType::Modulo
                ) {
                    expr.set_type(right_ty.clone());
                    self.current_type = Some(right_ty.clone());
                } else {
                    expr.set_type(Type::Bool);
                    self.current_type = Some(Type::Bool);
                }

                Ok(())
            }
            // Unary operation
            None => match expr.op {
                OpType::Minus => {
                    self.visit_expression(&mut expr.left)?;
                    expr.set_type(
                        self.current_type
                            .as_ref()
                            .expect("Expression has no type")
                            .clone(),
                    );
                    Ok(())
                }
                OpType::Not => {
                    self.visit_expression(&mut expr.left)?;
                    self.current_type = Some(Type::Bool);
                    expr.set_type(Type::Bool);
                    Ok(())
                }
                // This is a bug, and should never happen
                _ => unreachable!("Unary operation should be `not` or `-`"),
            },
        }
    }

    fn visit_literal(&mut self, literal: &'ast mut Literal) -> Result<(), TypeCheckerError> {
        match &literal.literal_type {
            LiteralType::True | LiteralType::False => {
                self.current_type = Some(Type::Bool);
                literal.set_type(Type::Bool);
            }
            LiteralType::Integer(_) => {
                self.current_type = Some(Type::Int);
                literal.set_type(Type::Int);
            }
            LiteralType::Float(_) => {
                self.current_type = Some(Type::Float);
                literal.set_type(Type::Float);
            }
            LiteralType::String(_) => {
                self.current_type = Some(Type::String);
                literal.set_type(Type::String);
            }
            LiteralType::Identifier(_) => {
                // FIXME: This is ugly and should not be written this way. We're
                // cloning here to trick the borrow checker and do mutable accept
                match literal.get_definition().clone() {
                    Definition::Struct(_) => {
                        let strct = literal.get_struct_def();
                        // self.visit_struct(strct)?;
                        self.current_type = Some(strct.get_type().clone());
                        literal.set_type(strct.get_type().clone());
                    }
                    Definition::LocalVariable(_) => {
                        self.current_type =
                            Some(literal.get_local_variable_def().get_type().clone());
                        literal.set_type(literal.get_local_variable_def().get_type().clone());
                    }
                    Definition::Function(_) => {
                        self.current_type = Some(literal.get_function_def().get_type().clone());
                        literal.set_type(literal.get_function_def().get_type().clone());
                    }
                }
            }
            LiteralType::ArrayAccess(_) => {
                let ty = match literal.get_definition() {
                    Definition::Struct(_) => unreachable!(),
                    Definition::LocalVariable(_) => {
                        literal.get_local_variable_def().get_type().clone()
                    }
                    Definition::Function(_) => {
                        if let Type::Function { return_type, .. } =
                            literal.get_function_def().get_type()
                        {
                            return_type.clone().deref().to_owned()
                        } else {
                            unreachable!()
                        }
                    }
                };
                match ty {
                    Type::Array { array_type, .. } => {
                        literal.set_type(array_type.clone().deref().to_owned());
                    }
                    _ => return Err(TypeCheckerError::NonSubscriptable { ty }),
                }
            }
            LiteralType::Null(_) => {
                self.current_type = Some(Type::Null {
                    concrete_type: None,
                });

                // No need to go further.
                return Ok(());
            }
        };

        // The identifier type should be the array type. We need to do it
        // here to avoid mutable while the literal is borrowed as ummutable
        let literal_ty = literal.get_type().clone();
        if let LiteralType::ArrayAccess(aa) = &mut literal.literal_type {
            aa.set_type(literal_ty.clone());
            let mut setter = ExpressionTypeSetter::new(&literal_ty);
            setter.set_type_recusively(&mut aa.identifier);

            // Index type must be set to int64.
            // TODO: Get pointer size type on targeted platform
            self.visit_expression(&mut aa.index)?;
            let index_type = self.current_type.as_ref().expect("Should have a type");
            if !index_type.is_integer() {
                return Err(TypeCheckerError::IndexNotInteger {
                    got: index_type.clone(),
                });
            }

            // Restore array accesss type back
            self.current_type = Some(literal.get_type().clone());
        }

        Ok(())
    }

    fn visit_array_initializer(
        &mut self,
        expr: &'ast mut ArrayInitializer,
    ) -> Result<(), TypeCheckerError> {
        let first_type = match expr.values.first_mut() {
            Some(exp) => {
                self.visit_expression(exp.as_mut())?;
                self.current_type.clone().expect("Expression has no type")
            }
            None => todo!("Handle empty array init"),
        };

        for (i, exp) in expr.values.iter_mut().enumerate() {
            self.visit_expression(exp)?;

            if self
                .current_type
                .as_ref()
                .expect("Array expression has no type")
                != &first_type
            {
                return Err(TypeCheckerError::DifferentTypeInArrayInitializer {
                    first: first_type,
                    found: self.current_type.clone().unwrap(),
                    position: i as u32,
                });
            }
        }

        expr.set_type(Type::Array {
            size: expr.values.len() as u32,
            array_type: Box::new(first_type.clone()),
        });

        self.current_type = Some(Type::Array {
            size: expr.values.len() as u32,
            array_type: Box::new(first_type),
        });
        Ok(())
    }

    fn visit_addrof(&mut self, expr: &'ast mut AddrOf) -> Result<(), TypeCheckerError> {
        self.visit_expression(&mut expr.expr)?;

        self.current_type = Some(Type::Ptr(Box::new(
            self.current_type.clone().expect("Should have a type"),
        )));

        Ok(())
    }

    fn visit_deref(&mut self, expr: &'ast mut ast::Deref) -> Result<(), TypeCheckerError> {
        self.visit_expression(&mut expr.expr)?;

        match self.current_type.as_ref().expect("Should have a type") {
            Type::Ptr(pointee) => {
                self.current_type = Some(pointee.deref().to_owned());
                Ok(())
            }
            _ => Err(TypeCheckerError::DerefNonPointer(
                self.current_type.clone().unwrap(),
            )),
        }
    }
}
