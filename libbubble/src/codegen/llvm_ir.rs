use inkwell::{
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
    values::{
        AnyValue, AnyValueEnum, BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue,
    },
    AddressSpace, FloatPredicate, IntPredicate,
};
use std::{collections::HashMap, convert::Infallible};

use crate::{
    ast::{
        Assignment, BinaryOperation, BreakStatement, Call, ForStatement, FunctionStatement,
        IfStatement, LetStatement, Literal, OpType, ReturnStatement, StructStatement, Visitor,
        WhileStatement,
    },
    codegen::locals_collector::SymbolsMap,
    type_system::{self, Type},
};

pub struct Translator<'ctx, 'ast> {
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    module: Module<'ctx>,
    frame_table: &'ast SymbolsMap<'ast>,
    variables: HashMap<&'ast str, PointerValue<'ctx>>,
    current_fn_value: Option<FunctionValue<'ctx>>,
    current_value: Option<AnyValueEnum<'ctx>>,
}

impl<'ctx, 'ast> Translator<'ctx, 'ast> {
    pub fn new(
        context: &'ctx Context,
        builder: &'ctx Builder<'ctx>,
        module: Module<'ctx>,
        frame_table: &'ast SymbolsMap<'ast>,
    ) -> Self {
        Self {
            context,
            builder,
            module,
            frame_table,
            variables: HashMap::new(),
            current_fn_value: None,
            current_value: None,
        }
    }

    #[inline]
    fn as_basic_type(&self, ty: AnyTypeEnum<'ctx>) -> BasicTypeEnum<'ctx> {
        match ty {
            AnyTypeEnum::ArrayType(x) => x.as_basic_type_enum(),
            AnyTypeEnum::FloatType(x) => x.as_basic_type_enum(),
            AnyTypeEnum::IntType(x) => x.as_basic_type_enum(),
            AnyTypeEnum::PointerType(x) => x.as_basic_type_enum(),
            AnyTypeEnum::StructType(x) => x.as_basic_type_enum(),
            AnyTypeEnum::VectorType(x) => x.as_basic_type_enum(),
            _ => panic!("Non basic type!"),
        }
    }

    #[inline]
    fn as_basic_value(&self, value: AnyValueEnum<'ctx>) -> BasicValueEnum<'ctx> {
        match value {
            AnyValueEnum::ArrayValue(v) => BasicValueEnum::ArrayValue(v),
            AnyValueEnum::IntValue(v) => BasicValueEnum::IntValue(v),
            AnyValueEnum::FloatValue(v) => BasicValueEnum::FloatValue(v),
            AnyValueEnum::PointerValue(v) => BasicValueEnum::PointerValue(v),
            AnyValueEnum::StructValue(v) => BasicValueEnum::StructValue(v),
            AnyValueEnum::VectorValue(v) => BasicValueEnum::VectorValue(v),
            _ => panic!("value is not basic!"),
        }
    }

    fn to_llvm_type(&self, ty: &type_system::Type) -> AnyTypeEnum<'ctx> {
        match ty {
            type_system::Type::U8 | type_system::Type::I8 => self.context.i8_type().into(),
            type_system::Type::U16 | type_system::Type::I16 => self.context.i16_type().into(),
            type_system::Type::U32 | type_system::Type::I32 => self.context.i32_type().into(),
            type_system::Type::U64 | type_system::Type::I64 => self.context.i64_type().into(),
            type_system::Type::Int => unreachable!(),
            type_system::Type::Float => self.context.f64_type().into(),
            type_system::Type::String => self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into(),
            type_system::Type::Bool => self.context.bool_type().into(),
            type_system::Type::Struct { .. } => todo!(),
            type_system::Type::Function {
                parameters,
                return_type,
            } => {
                let ret = self.as_basic_type(self.to_llvm_type(return_type));
                let param_ty: Vec<BasicMetadataTypeEnum> = parameters
                    .iter()
                    .map(|(kind, _)| self.as_basic_type(self.to_llvm_type(kind)).into())
                    .collect();

                ret.fn_type(&param_ty, false).into()
            }
            type_system::Type::Void => self.context.void_type().into(),
        }
    }

    #[inline]
    fn get_fn_value(&self) -> &FunctionValue<'ctx> {
        self.current_fn_value
            .as_ref()
            .expect("current_fn_value is None!")
    }

    fn create_entry_block_alloca<T: BasicType<'ctx>>(
        &self,
        name: &str,
        ty: T,
    ) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry = self
            .get_fn_value()
            .get_first_basic_block()
            .expect("Fn has no basic block!");

        match entry.get_first_instruction() {
            Some(ref instr) => builder.position_before(instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(ty, name)
    }
}

impl<'ast, 'ctx> Visitor<'ast, Infallible> for Translator<'ctx, 'ast> {
    fn visit_function(&mut self, stmt: &'ast FunctionStatement) -> Result<(), Infallible> {
        let (return_type, parameters) = if let Type::Function {
            return_type,
            parameters,
        } = stmt.ty.as_ref().expect("Function has no type")
        {
            (return_type, parameters)
        } else {
            panic!("Function type isn't a function type!")
        };

        let llvm_return_type = self.as_basic_type(self.to_llvm_type(return_type));
        let llvm_parameters_type: Vec<BasicMetadataTypeEnum<'ctx>> = parameters
            .iter()
            .map(|(ty, _)| {
                self.as_basic_type(self.to_llvm_type(ty))
                    .as_basic_type_enum()
                    .into()
            })
            .collect();

        let fn_ty = llvm_return_type.fn_type(&llvm_parameters_type, false);
        let fn_val = self
            .module
            .add_function(&stmt.name, fn_ty, Some(Linkage::External));

        for (i, arg) in fn_val.get_param_iter().enumerate() {
            arg.set_name(&stmt.parameters[i].1);
        }

        // Function parameter and local variables allocas
        for stack_var in self
            .frame_table
            .get(stmt.name.as_str())
            .expect("Function not collected!")
            .iter()
        {
            self.variables.insert(
                stack_var.name,
                self.create_entry_block_alloca(
                    stack_var.name,
                    self.as_basic_type(self.to_llvm_type(stack_var.kind)),
                ),
            );
        }

        Ok(())
    }

    fn visit_struct(&mut self, _: &'ast StructStatement) -> Result<(), Infallible> {
        todo!("Implement struct!")
    }

    fn visit_let(&mut self, stmt: &'ast LetStatement) -> Result<(), Infallible> {
        self.visit_expression(&stmt.init_exp)?;

        let store_value = self
            .variables
            .get(stmt.name.as_str())
            .expect("Variable does not exist!");

        self.builder.build_store(
            *store_value,
            self.current_value.unwrap().into_pointer_value(),
        );

        self.current_value = Some(store_value.as_any_value_enum());
        self.variables.insert(&stmt.name, *store_value);

        Ok(())
    }

    fn visit_if(&mut self, stmt: &'ast IfStatement) -> Result<(), Infallible> {
        let parent = self.current_fn_value.unwrap();
        let zero_const = self.context.i64_type().const_zero();

        self.visit_expression(&stmt.condition)?;
        let condition = self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            zero_const,
            self.current_value.unwrap().into_int_value(),
            "if_condition",
        );

        let then_bb = self.context.append_basic_block(parent, "then");
        let else_bb = self.context.append_basic_block(parent, "then");
        let merge_bb = self.context.append_basic_block(parent, "then");

        self.builder
            .build_conditional_branch(condition, then_bb, else_bb);

        self.builder.position_at_end(then_bb);
        self.visit_statements(&stmt.then_clause)?;
        self.builder.build_unconditional_branch(merge_bb);

        if let Some(ref stmts) = stmt.else_clause {
            self.builder.position_at_end(else_bb);
            self.visit_statements(stmts)?;
            self.builder.build_unconditional_branch(merge_bb);

            self.builder.get_insert_block().unwrap();
        }

        self.builder.position_at_end(merge_bb);
        Ok(())
    }

    fn visit_while(&mut self, stmt: &'ast WhileStatement) -> Result<(), Infallible> {
        let parent = self.current_fn_value.unwrap();
        let zero_const = self.context.i64_type().const_zero();

        let while_block = self.context.append_basic_block(parent, "while");
        let after_while_block = self.context.append_basic_block(parent, "after_while");
        self.builder.position_at_end(while_block);
        self.visit_expression(&stmt.condition)?;

        let condition = self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            zero_const,
            self.current_value.unwrap().into_int_value(),
            "if_condition",
        );
        self.builder
            .build_conditional_branch(condition, while_block, after_while_block);
        self.visit_statements(&stmt.body)?;
        self.builder.build_unconditional_branch(while_block); // Loop

        Ok(())
    }

    fn visit_for(&mut self, stmt: &'ast ForStatement) -> Result<(), Infallible> {
        todo!("for desugar")
    }

    fn visit_return(&mut self, stmt: &'ast ReturnStatement) -> Result<(), Infallible> {
        self.visit_expression(&stmt.exp)?;

        // FIXME: Very ugly way to retrieve a &dyn BasicValue handle from a AnyValueEnum
        self.builder
            .build_return(Some(match self.current_value.as_ref().unwrap() {
                AnyValueEnum::ArrayValue(v) => v,
                AnyValueEnum::IntValue(v) => v,
                AnyValueEnum::FloatValue(v) => v,
                AnyValueEnum::PointerValue(v) => v,
                AnyValueEnum::StructValue(v) => v,
                AnyValueEnum::VectorValue(v) => v,
                _ => panic!("value is not basic!"),
            }));

        Ok(())
    }

    fn visit_break(&mut self, stmt: &'ast BreakStatement) -> Result<(), Infallible> {
        todo!()
    }

    fn visit_binary_operation(&mut self, expr: &'ast BinaryOperation) -> Result<(), Infallible> {
        self.visit_expression(&expr.left)?;
        let left = self.current_value.unwrap();

        // Unary operation
        if expr.right.is_none() {
            self.current_value = Some(match expr.op {
                OpType::Not => self.builder.build_not(left.into_int_value(), "not").into(),
                OpType::Minus => match left {
                    AnyValueEnum::IntValue(v) => self.builder.build_int_neg(v, "neg").into(),
                    AnyValueEnum::FloatValue(v) => {
                        self.builder.build_float_neg(v, "neg_float").into()
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            });

            return Ok(());
        }

        self.visit_expression(expr.right.as_ref().unwrap())?;
        let right = self.current_value.unwrap();

        let value: AnyValueEnum = match expr.op {
            OpType::And => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => {
                    self.builder.build_and(v1, v2, "and").into()
                }
                _ => unreachable!(),
            },
            OpType::Different => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_compare(IntPredicate::NE, v1, v2, "!=")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Divide => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => {
                    self.builder.build_int_signed_div(v1, v2, "divide").into()
                }
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => {
                    self.builder.build_float_div(v1, v2, "divide_float").into()
                }
                _ => unreachable!(),
            },
            OpType::Equal => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_compare(IntPredicate::EQ, v1, v2, "equal")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_compare(FloatPredicate::OEQ, v1, v2, "divide_float")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Less => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_compare(IntPredicate::SLT, v1, v2, "equal")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_compare(FloatPredicate::OLT, v1, v2, "divide_float")
                    .into(),
                _ => unreachable!(),
            },
            OpType::LessEqual => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_compare(IntPredicate::SLE, v1, v2, "less_equal")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_compare(FloatPredicate::OLE, v1, v2, "less_equal_float")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Minus => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => {
                    self.builder.build_int_sub(v1, v2, "sub").into()
                }
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => {
                    self.builder.build_float_sub(v1, v2, "sub_float").into()
                }
                _ => unreachable!(),
            },
            OpType::Modulo => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => {
                    self.builder.build_int_signed_rem(v1, v2, "modulo").into()
                }
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => {
                    self.builder.build_float_rem(v1, v2, "sub_float").into()
                }
                _ => unreachable!(),
            },
            OpType::More => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_compare(IntPredicate::SGT, v1, v2, "equal")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_compare(FloatPredicate::OGT, v1, v2, "divide_float")
                    .into(),
                _ => unreachable!(),
            },
            OpType::MoreEqual => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_compare(IntPredicate::SGE, v1, v2, "equal")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_compare(FloatPredicate::OGE, v1, v2, "divide_float")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Multiply => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => {
                    self.builder.build_int_mul(v1, v2, "equal").into()
                }
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => {
                    self.builder.build_float_mul(v1, v2, "divide_float").into()
                }
                _ => unreachable!(),
            },
            OpType::Or => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => {
                    self.builder.build_or(v1, v2, "or").into()
                }
                _ => unreachable!(),
            },
            OpType::Plus => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => {
                    self.builder.build_int_add(v1, v2, "add").into()
                }
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => {
                    self.builder.build_float_add(v1, v2, "add").into()
                }
                _ => unreachable!(),
            },
            OpType::Not => unreachable!("not isn't a binary operation"),
        };

        self.current_value = Some(value);

        Ok(())
    }

    fn visit_literal(&mut self, stmt: &'ast Literal) -> Result<(), Infallible> {
        match &stmt.literal_type {
            crate::ast::LiteralType::True => {
                self.current_value = Some(self.context.bool_type().const_int(1, false).into());
            }
            crate::ast::LiteralType::False => {
                self.current_value = Some(self.context.bool_type().const_zero().into());
            }
            crate::ast::LiteralType::Integer(x) => {
                self.current_value = Some(self.context.i64_type().const_int(*x as u64, true).into())
            }
            crate::ast::LiteralType::Float(x) => {
                self.current_value = Some(self.context.f64_type().const_float(*x).into())
            }
            crate::ast::LiteralType::Identifier(id) => {
                self.current_value = Some(
                    self.variables
                        .get(id.as_str())
                        .expect("variable not found!")
                        .as_any_value_enum(),
                );
            }
        }

        Ok(())
    }

    fn visit_call(&mut self, expr: &'ast Call) -> Result<(), Infallible> {
        let mut parameters_values: Vec<BasicMetadataValueEnum<'_>> =
            Vec::with_capacity(expr.arguments.len());
        let fn_value = self
            .module
            .get_function(&expr.callee)
            .expect("Function not found");

        for arg in &expr.arguments {
            self.visit_expression(arg)?;
            parameters_values.push(self.as_basic_value(self.current_value.unwrap()).into());
        }

        self.current_value = Some(
            self.builder
                .build_call(fn_value, &parameters_values, "call")
                .as_any_value_enum(),
        );

        Ok(())
    }

    fn visit_assignment(&mut self, expr: &'ast Assignment) -> Result<(), Infallible> {
        self.visit_expression(&expr.left)?;
        let lhs = self.current_value.unwrap();
        self.visit_expression(&expr.right)?;
        let rhs = self.current_value.unwrap();

        self.builder
            .build_store(lhs.into_pointer_value(), self.as_basic_value(rhs));

        Ok(())
    }
}
