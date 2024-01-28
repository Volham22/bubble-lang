use inkwell::{
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
    values::{
        AnyValue, AnyValueEnum, BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue,
        PointerValue,
    },
    AddressSpace, FloatPredicate, IntPredicate,
};
use std::{collections::HashMap, convert::Infallible};

use crate::{
    ast::{
        ArrayInitializer, Assignment, BinaryOperation, BreakStatement, Call, Expression,
        ForStatement, FunctionStatement, GlobalStatement, IfStatement, LetStatement, Literal,
        LiteralType, OpType, ReturnStatement, StructStatement, Visitor, WhileStatement,
    },
    codegen::locals_collector::SymbolsMap,
    type_system::{self, Typable, Type},
};

use super::Collector;

pub fn build_module<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    statements: &[GlobalStatement],
) {
    let mut frame_table = Collector::default();
    // Collect local variables and function parameters
    let symbol_map = frame_table.dump_global_statements(statements).unwrap();

    let builder = context.create_builder();

    let mut translator = Translator::new(context, builder, module, symbol_map);
    translator.translate_statements(statements).unwrap();
    translator.print_code();
}

pub struct Translator<'ctx, 'ast, 'module> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: &'module Module<'ctx>,
    frame_table: &'ast SymbolsMap<'ast>,
    variables: HashMap<&'ast str, PointerValue<'ctx>>,
    current_fn_value: Option<FunctionValue<'ctx>>,
    current_value: Option<AnyValueEnum<'ctx>>,
}

impl<'ctx, 'ast, 'module> Translator<'ctx, 'ast, 'module> {
    pub fn new(
        context: &'ctx Context,
        builder: Builder<'ctx>,
        module: &'module Module<'ctx>,
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

    pub fn translate_statements(
        &mut self,
        stmts: &'ast [GlobalStatement],
    ) -> Result<(), Infallible> {
        for stmt in stmts {
            self.visit_global_statement(stmt)?;
        }

        if let Err(msg) = self.module.verify() {
            self.print_code();
            eprintln!("Failed to verify module!\n{}", msg.to_string());
        }

        Ok(())
    }

    pub fn print_code(&self) {
        let content = self.module.print_to_string().to_string();
        println!(
            "==== LLVM IR of module '{}' ====",
            self.module.get_name().to_str().unwrap()
        );

        println!("{}", content);
        println!("==============================")
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
            type_system::Type::Array { size, array_type } => {
                let base_type = self.to_llvm_type(array_type);

                match base_type {
                    AnyTypeEnum::ArrayType(t) => t.array_type(*size).into(),
                    AnyTypeEnum::FloatType(t) => t.array_type(*size).into(),
                    AnyTypeEnum::IntType(t) => t.array_type(*size).into(),
                    AnyTypeEnum::PointerType(t) => t.array_type(*size).into(),
                    AnyTypeEnum::StructType(t) => t.array_type(*size).into(),
                    AnyTypeEnum::VectorType(t) => t.array_type(*size).into(),
                    _ => unreachable!("Type couldn't be an array!"),
                }
            }
            type_system::Type::Ptr(pointee) => {
                match self.as_basic_type(self.to_llvm_type(pointee)) {
                    BasicTypeEnum::ArrayType(t) => t.ptr_type(AddressSpace::from(0u16)).into(),
                    BasicTypeEnum::FloatType(t) => t.ptr_type(AddressSpace::from(0u16)).into(),
                    BasicTypeEnum::IntType(t) => t.ptr_type(AddressSpace::from(0u16)).into(),
                    BasicTypeEnum::PointerType(t) => t.ptr_type(AddressSpace::from(0u16)).into(),
                    BasicTypeEnum::StructType(t) => t.ptr_type(AddressSpace::from(0u16)).into(),
                    BasicTypeEnum::VectorType(t) => t.ptr_type(AddressSpace::from(0u16)).into(),
                }
            }
            type_system::Type::Null { concrete_type } => self.to_llvm_type(concrete_type.as_ref().expect("Should have a concrete type")),
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

        builder
            .build_alloca(ty, name)
            .expect("failed to build alloca")
    }
}

impl<'ast, 'ctx, 'module> Visitor<'ast, Infallible> for Translator<'ctx, 'ast, 'module> {
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

        let llvm_parameters_type: Vec<BasicMetadataTypeEnum<'ctx>> = parameters
            .iter()
            .map(|(ty, _)| {
                self.as_basic_type(self.to_llvm_type(ty))
                    .as_basic_type_enum()
                    .into()
            })
            .collect();

        let fn_ty = if *return_type.as_ref() != type_system::Type::Void {
            self.as_basic_type(self.to_llvm_type(return_type))
                .fn_type(&llvm_parameters_type, false)
        } else {
            self.context
                .void_type()
                .fn_type(&llvm_parameters_type, false)
        };

        let fn_val = self.module.add_function(
            &stmt.name,
            fn_ty,
            Some(if stmt.body.is_some() {
                // We don't want external function to be exported
                Linkage::External
            } else {
                Linkage::ExternalWeak
            }),
        );

        // Stop function generation here it's an extern declaration
        if stmt.body.is_none() {
            self.current_fn_value = None;
            return Ok(());
        }

        self.current_fn_value = Some(fn_val);
        let entry = self.context.append_basic_block(fn_val, &stmt.name);
        self.builder.position_at_end(entry);

        for (i, arg) in fn_val.get_param_iter().enumerate() {
            let arg_name = &stmt.parameters[i].name;
            arg.set_name(&stmt.parameters[i].name);
            let alloca = self.create_entry_block_alloca(arg_name, arg.get_type());
            self.builder
                .build_store(alloca, arg)
                .expect("Fail to build store");
            self.variables.insert(arg_name, alloca);
        }

        // local variables allocas
        for stack_var in self
            .frame_table
            .get(stmt.name.as_str())
            .expect("Function not collected!")
            .iter()
        {
            let alloca = self.create_entry_block_alloca(
                stack_var.name,
                self.as_basic_type(self.to_llvm_type(stack_var.kind)),
            );
            self.variables.insert(stack_var.name, alloca);
        }

        self.visit_statements(stmt.body.as_ref().unwrap())?;
        self.current_fn_value = None;

        Ok(())
    }

    fn visit_struct(&mut self, _: &'ast StructStatement) -> Result<(), Infallible> {
        todo!("Implement struct!")
    }

    fn visit_let(&mut self, stmt: &'ast LetStatement) -> Result<(), Infallible> {
        self.visit_expression(
            stmt.init_exp
                .as_ref()
                .expect("Let statement has no init exp"),
        )?;

        if let Type::Array { array_type, .. } = stmt.get_type() {
            let Expression::ArrayInitializer(ArrayInitializer { values, .. }) = stmt
                .init_exp
                .as_ref()
                .expect("Array must have init expr")
                .as_ref()
            else {
                unreachable!("Array variable initializer is not an array initializer");
            };

            let store_value = *self
                .variables
                .get(stmt.name.as_str())
                .expect("Variable does not exist!");

            let pointee_type = self.to_llvm_type(array_type);

            for (i, exp) in values.iter().enumerate() {
                self.visit_expression(exp)?;

                let ptr_offset = unsafe {
                    self.builder
                        .build_gep(
                            self.as_basic_type(pointee_type),
                            store_value,
                            &[self.context.i64_type().const_int(i as u64, false)],
                            "array_store_init",
                        )
                        .expect("Fail to build array init GEP")
                };

                self.builder
                    .build_store(
                        ptr_offset,
                        self.as_basic_value(
                            self.current_value.expect("Array expression has no value"),
                        ),
                    )
                    .expect("Fail to build array init store");
            }
        } else {
            let store_value = self
                .variables
                .get(stmt.name.as_str())
                .expect("Variable does not exist!");

            self.builder
                .build_store(
                    *store_value,
                    match self.current_value.unwrap() {
                        AnyValueEnum::ArrayValue(v) => v.as_basic_value_enum(),
                        AnyValueEnum::IntValue(v) => v.as_basic_value_enum(),
                        AnyValueEnum::FloatValue(v) => v.as_basic_value_enum(),
                        AnyValueEnum::PointerValue(v) => v.as_basic_value_enum(),
                        AnyValueEnum::StructValue(v) => v.as_basic_value_enum(),
                        AnyValueEnum::VectorValue(v) => v.as_basic_value_enum(),
                        _ => unreachable!(),
                    },
                )
                .expect("Fail to build store");

            self.current_value = Some(store_value.as_any_value_enum());
            self.variables.insert(&stmt.name, *store_value);
        }

        Ok(())
    }

    fn visit_if(&mut self, stmt: &'ast IfStatement) -> Result<(), Infallible> {
        let parent = self.current_fn_value.unwrap();
        let zero_const = self.context.i64_type().const_zero();

        self.visit_expression(&stmt.condition)?;
        let condition = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                zero_const,
                self.current_value.unwrap().into_int_value(),
                "if_condition",
            )
            .expect("Fail to build int compare");

        let then_bb = self.context.append_basic_block(parent, "then");
        let else_bb = self.context.append_basic_block(parent, "else");
        let merge_bb = self.context.append_basic_block(parent, "merge");

        self.builder
            .build_conditional_branch(condition, then_bb, else_bb)
            .expect("Fail to build conditional branch");

        self.builder.position_at_end(then_bb);
        self.visit_statements(&stmt.then_clause)?;
        self.builder
            .build_unconditional_branch(merge_bb)
            .expect("Fail to build unconditional branch");

        self.builder.position_at_end(else_bb);
        if let Some(ref stmts) = stmt.else_clause {
            self.visit_statements(stmts)?;
        }

        self.builder
            .build_unconditional_branch(merge_bb)
            .expect("Fail to build unconditional branch");
        self.builder.position_at_end(merge_bb);
        Ok(())
    }

    fn visit_while(&mut self, stmt: &'ast WhileStatement) -> Result<(), Infallible> {
        let parent = self.current_fn_value.unwrap();
        let zero_const = self.context.bool_type().const_zero();

        let condition_block = self.context.append_basic_block(parent, "while_test");
        let while_block = self.context.append_basic_block(parent, "while_body");
        let after_while_block = self.context.append_basic_block(parent, "after_while");

        self.builder
            .build_unconditional_branch(condition_block)
            .expect("Fail to build unconditional branch");
        self.builder.position_at_end(condition_block);
        self.visit_expression(&stmt.condition)?;

        let condition = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                zero_const,
                self.current_value.unwrap().into_int_value(),
                "if_condition",
            )
            .expect("Fail to build int compare");
        self.builder
            .build_conditional_branch(condition, while_block, after_while_block)
            .expect("Fail to build unconditional branch");

        self.builder.position_at_end(while_block);
        self.visit_statements(&stmt.body)?;
        self.builder
            .build_unconditional_branch(condition_block) // Loop
            .expect("Fail to build unconditional branch");

        self.builder.position_at_end(after_while_block);

        Ok(())
    }

    fn visit_for(&mut self, _stmt: &'ast ForStatement) -> Result<(), Infallible> {
        unreachable!("for desugar")
    }

    fn visit_return(&mut self, stmt: &'ast ReturnStatement) -> Result<(), Infallible> {
        if let Some(ref exp) = stmt.exp {
            self.visit_expression(exp)?;

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
                }))
                .expect("Fail to build return");
        } else {
            self.builder
                .build_return(None)
                .expect("Fail to build return");
        }

        Ok(())
    }

    fn visit_break(&mut self, _stmt: &'ast BreakStatement) -> Result<(), Infallible> {
        todo!("implement break")
    }

    fn visit_binary_operation(&mut self, expr: &'ast BinaryOperation) -> Result<(), Infallible> {
        self.visit_expression(&expr.left)?;
        let left = self.current_value.unwrap();

        // Unary operation
        if expr.right.is_none() {
            self.current_value = Some(match expr.op {
                OpType::Not => self
                    .builder
                    .build_not(left.into_int_value(), "not")
                    .expect("Fail to build not")
                    .into(),
                OpType::Minus => match left {
                    AnyValueEnum::IntValue(v) => self
                        .builder
                        .build_int_neg(v, "neg")
                        .expect("Fail to build int neg")
                        .into(),
                    AnyValueEnum::FloatValue(v) => self
                        .builder
                        .build_float_neg(v, "neg_float")
                        .expect("Failed to build float neg")
                        .into(),
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
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_and(v1, v2, "and")
                    .expect("Fail to build and")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Different => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_compare(IntPredicate::NE, v1, v2, "!=")
                    .expect("Fail to build int compare")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Divide => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_signed_div(v1, v2, "divide")
                    .expect("Fail to build int signed div")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_div(v1, v2, "divide_float")
                    .expect("Fail to build float div")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Equal => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_compare(IntPredicate::EQ, v1, v2, "equal")
                    .expect("Fail to build int compare")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_compare(FloatPredicate::OEQ, v1, v2, "divide_float")
                    .expect("Fail to build float compare")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Less => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_compare(IntPredicate::SLT, v1, v2, "equal")
                    .expect("Fail to build int compare")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_compare(FloatPredicate::OLT, v1, v2, "divide_float")
                    .expect("Fail to build float compare")
                    .into(),
                _ => unreachable!(),
            },
            OpType::LessEqual => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_compare(IntPredicate::SLE, v1, v2, "less_equal")
                    .expect("Fail to build int compare")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_compare(FloatPredicate::OLE, v1, v2, "less_equal_float")
                    .expect("Fail to build float compare")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Minus => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_sub(v1, v2, "sub")
                    .expect("Fail to build int sub")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_sub(v1, v2, "sub_float")
                    .expect("Fail to build float sub")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Modulo => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_signed_rem(v1, v2, "modulo")
                    .expect("Fail to build int signed rem")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_rem(v1, v2, "sub_float")
                    .expect("Fail to build float rem")
                    .into(),
                _ => unreachable!(),
            },
            OpType::More => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_compare(IntPredicate::SGT, v1, v2, "equal")
                    .expect("Fail to build int compare")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_compare(FloatPredicate::OGT, v1, v2, "divide_float")
                    .expect("Fail to build float compare")
                    .into(),
                _ => unreachable!(),
            },
            OpType::MoreEqual => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_compare(IntPredicate::SGE, v1, v2, "equal")
                    .expect("Fail to build int compare")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_compare(FloatPredicate::OGE, v1, v2, "divide_float")
                    .expect("Fail to build float compare")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Multiply => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_mul(v1, v2, "equal")
                    .expect("Fail to build int mul")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_mul(v1, v2, "divide_float")
                    .expect("Fail to build float mul")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Or => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_or(v1, v2, "or")
                    .expect("Fail to build or")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Plus => match (left, right) {
                (AnyValueEnum::IntValue(v1), AnyValueEnum::IntValue(v2)) => self
                    .builder
                    .build_int_add(v1, v2, "add")
                    .expect("Fail to build int add")
                    .into(),
                (AnyValueEnum::FloatValue(v1), AnyValueEnum::FloatValue(v2)) => self
                    .builder
                    .build_float_add(v1, v2, "add")
                    .expect("Fail to build float add")
                    .into(),
                _ => unreachable!(),
            },
            OpType::Not => unreachable!("not isn't a binary operation"),
        };

        self.current_value = Some(value);

        Ok(())
    }

    fn visit_literal(&mut self, stmt: &'ast Literal) -> Result<(), Infallible> {
        match &stmt.literal_type {
            LiteralType::True => {
                self.current_value = Some(self.context.bool_type().const_int(1, false).into());
            }
            LiteralType::False => {
                self.current_value = Some(self.context.bool_type().const_zero().into());
            }
            LiteralType::Integer(x) => {
                let int_ty = stmt
                    .ty
                    .as_ref()
                    .expect("integer like literal must have type!");

                self.current_value = Some(
                    self.to_llvm_type(int_ty)
                        .into_int_type()
                        .const_int(*x as u64, int_ty.is_signed())
                        .into(),
                );
            }
            LiteralType::Float(x) => {
                self.current_value = Some(self.context.f64_type().const_float(*x).into())
            }
            LiteralType::Identifier(id) => {
                let ptr = self
                    .variables
                    .get(id.as_str())
                    .expect("variable not found!");

                println!("Statement: {:?}", stmt);
                self.current_value = Some(
                    self.builder
                        .build_load(
                            self.as_basic_type(self.to_llvm_type(stmt.get_type())),
                            *ptr,
                            "load",
                        )
                        .expect("Fail to build load")
                        .as_any_value_enum(),
                );
            }
            LiteralType::String(content) => {
                self.current_value = Some(
                    self.builder
                        .build_global_string_ptr(content.as_str(), "string_literal")
                        .expect("Fail to build global string ptr")
                        .as_any_value_enum(),
                );
            }
            LiteralType::ArrayAccess(array_access) => {
                // Translate and get array llvm value (array ptr)
                self.visit_expression(&array_access.identifier)?;
                let pointee_ty = self.as_basic_type(self.to_llvm_type(array_access.get_type()));

                // If it's a literal we can't visit the expression because we need
                // a pointer like type. Visiting the expression would give us the pointee value
                let ptr_value = match array_access.identifier.as_ref() {
                    Expression::Literal(Literal {
                        literal_type: LiteralType::Identifier(name),
                        ..
                    }) => *self
                        .variables
                        .get(name.as_str())
                        .expect("Variable does not exist"),
                    _ => {
                        self.visit_expression(&array_access.identifier)?;
                        self.current_value
                            .as_ref()
                            .expect("Array access has no value")
                            .into_pointer_value()
                    }
                };

                // Translate and store index expression
                self.visit_expression(&array_access.index)?;
                let index_value = self
                    .current_value
                    .as_ref()
                    .expect("Array access index has no value")
                    .into_int_value();

                // Compute offset with getelementptr
                let load_ptr_value = unsafe {
                    self.builder
                        .build_gep(
                            pointee_ty,
                            ptr_value,
                            &[index_value],
                            "load_ptr_array_access",
                        )
                        .expect("Fail to build gep for array access")
                };

                // Load it as usual
                self.current_value = Some(
                    self.builder
                        .build_load(pointee_ty, load_ptr_value, "array_acess_load")
                        .expect("Fail to build array access load")
                        .into(),
                );
            }
            LiteralType::Null(n) => {
                let llvm_ty = self.to_llvm_type(n.get_type()).into_pointer_type();
                self.current_value = Some(llvm_ty.const_null().into());
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
                .expect("Fail to build call")
                .as_any_value_enum(),
        );

        Ok(())
    }

    fn visit_assignment(&mut self, expr: &'ast Assignment) -> Result<(), Infallible> {
        self.visit_expression(&expr.right)?;
        let rhs = self.current_value.unwrap();

        // TODO: Add an `as_lvalue` method for `Literal` to make it cleaner
        let lhs = self
            .variables
            .get(match expr.left.as_ref() {
                Expression::Literal(l) => match &l.literal_type {
                    LiteralType::Identifier(id) => id.as_str(),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            })
            .unwrap();

        self.builder
            .build_store(*lhs, self.as_basic_value(rhs))
            .expect("Fail to build store");

        Ok(())
    }
}
