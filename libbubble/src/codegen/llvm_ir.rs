use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicType},
    values::PointerValue,
    AddressSpace,
};
use std::{collections::HashMap, convert::Infallible};

use crate::{
    ast::{
        Assignment, BinaryOperation, BreakStatement, Call, ForStatement, FunctionStatement,
        IfStatement, LetStatement, Literal, ReturnStatement, StructStatement, Visitor,
        WhileStatement,
    },
    codegen::locals_collector::SymbolsMap,
    type_system,
};

pub struct Translator<'ctx, 'ast> {
    context: &'ctx Context,
    builder: &'ctx Builder<'ctx>,
    module: Module<'ctx>,
    frame_table: &'ast SymbolsMap<'ast>,
    variables: HashMap<String, PointerValue<'ctx>>,
    current_value: Option<PointerValue<'ctx>>,
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
            current_value: None,
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
                let as_basic_type = |ty: AnyTypeEnum<'ctx>| match ty {
                    AnyTypeEnum::ArrayType(x) => x.as_basic_type_enum(),
                    AnyTypeEnum::FloatType(x) => x.as_basic_type_enum(),
                    AnyTypeEnum::IntType(x) => x.as_basic_type_enum(),
                    AnyTypeEnum::PointerType(x) => x.as_basic_type_enum(),
                    AnyTypeEnum::StructType(x) => x.as_basic_type_enum(),
                    AnyTypeEnum::VectorType(x) => x.as_basic_type_enum(),
                    _ => panic!("Non basic type!"),
                };

                let ret = as_basic_type(self.to_llvm_type(return_type));
                let param_ty: Vec<BasicMetadataTypeEnum> = parameters
                    .iter()
                    .map(|(kind, _)| as_basic_type(self.to_llvm_type(kind)).into())
                    .collect();

                ret.fn_type(&param_ty, false).into()
            }
            type_system::Type::Void => self.context.void_type().into(),
        }
    }
}

impl<'ast, 'ctx> Visitor<'ast, Infallible> for Translator<'ctx, 'ast> {
    fn visit_function(&mut self, stmt: &'ast FunctionStatement) -> Result<(), Infallible> {
        todo!()
    }

    fn visit_struct(&mut self, _: &'ast StructStatement) -> Result<(), Infallible> {
        todo!("Implement struct!")
    }

    fn visit_let(&mut self, stmt: &'ast LetStatement) -> Result<(), Infallible> {
        todo!()
    }

    fn visit_if(&mut self, stmt: &'ast IfStatement) -> Result<(), Infallible> {
        todo!()
    }

    fn visit_while(&mut self, stmt: &'ast WhileStatement) -> Result<(), Infallible> {
        todo!()
    }

    fn visit_for(&mut self, stmt: &'ast ForStatement) -> Result<(), Infallible> {
        todo!()
    }

    fn visit_return(&mut self, stmt: &'ast ReturnStatement) -> Result<(), Infallible> {
        todo!()
    }

    fn visit_break(&mut self, _: &'ast BreakStatement) -> Result<(), Infallible> {
        todo!()
    }

    fn visit_binary_operation(&mut self, expr: &'ast BinaryOperation) -> Result<(), Infallible> {
        todo!()
    }

    fn visit_literal(&mut self, _: &'ast Literal) -> Result<(), Infallible> {
        todo!()
    }

    fn visit_call(&mut self, expr: &'ast Call) -> Result<(), Infallible> {
        todo!()
    }

    fn visit_assignment(&mut self, expr: &'ast Assignment) -> Result<(), Infallible> {
        todo!()
    }
}
