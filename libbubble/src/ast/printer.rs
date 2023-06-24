use std::io;

use super::{
    visitor::Visitor, Assignment, BinaryOperation, BreakStatement, Call, ContinueStatement,
    ForStatement, FunctionStatement, GlobalStatement, IfStatement, LetStatement, Literal,
    ReturnStatement, StructStatement, Type, TypeKind, WhileStatement,
};

pub struct Printer<Writer: io::Write> {
    indent_level: usize,
    writer: Writer,
}

pub type PrinterResult = Result<(), std::io::Error>;

impl std::default::Default for Printer<io::Stdout> {
    fn default() -> Self {
        Self {
            indent_level: Default::default(),
            writer: io::stdout(),
        }
    }
}

impl<T: io::Write> Printer<T> {
    pub fn print(&mut self, statements: Vec<GlobalStatement>) -> PrinterResult {
        for stmt in &statements {
            self.visit_global_statement(stmt)?;
        }

        Ok(())
    }

    fn write(&mut self, content: &str) -> Result<(), io::Error> {
        let indent_str = self.get_indent_str();
        let to_write = content.replace('\n', &format!("\n{indent_str}"));
        self.writer.write_all(to_write.as_bytes())?;

        Ok(())
    }

    fn indent_and_newline(&mut self) -> Result<(), io::Error> {
        self.indent_level += 1;
        self.writer.write_all(b"\n")?;
        self.writer.write_all(self.get_indent_str().as_bytes())?;
        Ok(())
    }

    fn dec_indent_and_newline(&mut self) -> Result<(), io::Error> {
        self.indent_level -= 1;
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    fn get_indent_str(&self) -> String {
        " ".repeat(self.indent_level * 2)
    }
}

impl<T: io::Write> Visitor<io::Error> for Printer<T> {
    fn visit_function(&mut self, stmt: &FunctionStatement) -> PrinterResult {
        self.write("function ")?;
        self.write(&stmt.name)?;

        self.write("(")?;
        for (kind, name) in &stmt.parameters {
            self.write(name)?;
            self.write(":")?;
            kind.accept(self)?;
            self.write(", ")?;
        }
        self.write("): ")?;

        stmt.return_type.accept(self)?;
        self.write(" {")?;
        self.indent_and_newline()?;

        for stmt in &stmt.body.statements {
            stmt.kind.accept(self)?;
        }

        self.dec_indent_and_newline()?;
        self.write("}\n")?;

        Ok(())
    }

    fn visit_struct(&mut self, stmt: &StructStatement) -> PrinterResult {
        self.write(&format!("struct {} {{", stmt.name))?;
        self.indent_and_newline()?;

        for (kind, name) in &stmt.fields {
            self.write(&format!("{}: ", name))?;
            kind.accept(self)?;
            self.write(",\n")?;
        }

        self.dec_indent_and_newline()?;
        self.write("}\n")
    }

    fn visit_let(&mut self, stmt: &LetStatement) -> PrinterResult {
        self.write("let ")?;
        self.write(&stmt.name)?;

        if let Some(ty) = &stmt.declaration_type {
            ty.accept(self)?;
        }

        self.write(" = ")?;
        stmt.init_exp.accept(self)?;
        self.write(";\n")?;

        Ok(())
    }

    fn visit_if(&mut self, stmt: &IfStatement) -> PrinterResult {
        self.write("if ")?;
        stmt.condition.accept(self)?;
        self.write("{")?;
        self.indent_and_newline()?;
        self.visit_statements(&stmt.then_clause)?;
        self.write("}")?;
        self.dec_indent_and_newline()?;

        if let Some(else_clause) = &stmt.else_clause {
            self.write("else {")?;
            self.indent_and_newline()?;
            self.visit_statements(else_clause)?;
            self.write("}")?;
            self.dec_indent_and_newline()?;
        }

        Ok(())
    }

    fn visit_while(&mut self, stmt: &WhileStatement) -> PrinterResult {
        self.write("while ")?;
        stmt.condition.accept(self)?;

        self.write(" {")?;
        self.indent_and_newline()?;
        self.visit_statements(&stmt.body)?;
        self.write("}")?;
        self.dec_indent_and_newline()?;

        Ok(())
    }

    fn visit_for(&mut self, stmt: &ForStatement) -> PrinterResult {
        self.write("for ")?;
        self.write(&stmt.init_decl.name)?;

        if let Some(ty) = &stmt.init_decl.declaration_type {
            self.write(": ")?;
            ty.accept(self)?;
        }

        self.write(" = ")?;
        stmt.init_decl.accept(self)?;
        self.write("; ")?;
        stmt.continue_expression.accept(self)?;
        self.write("; ")?;
        stmt.modify_expression.accept(self)?;

        self.write("{")?;
        self.indent_and_newline()?;
        self.visit_statements(&stmt.body)?;
        self.write("}")?;
        self.dec_indent_and_newline()?;

        Ok(())
    }

    fn visit_return(&mut self, stmt: &ReturnStatement) -> PrinterResult {
        self.write("return ")?;
        stmt.exp.accept(self)?;
        self.write(";")
    }

    fn visit_break(&mut self, _: &BreakStatement) -> PrinterResult {
        self.write("break;")
    }

    fn visit_continue(&mut self, _: &ContinueStatement) -> PrinterResult {
        self.write("continue;")
    }

    fn visit_binary_operation(&mut self, expr: &BinaryOperation) -> PrinterResult {
        expr.left.accept(self)?;

        match expr.op {
            super::OpType::And => self.write("and"),
            super::OpType::Different => self.write("!="),
            super::OpType::Divide => self.write("/"),
            super::OpType::Equal => self.write("=="),
            super::OpType::Less => self.write("<"),
            super::OpType::LessEqual => self.write("<="),
            super::OpType::Minus => self.write("-"),
            super::OpType::Modulo => self.write("%"),
            super::OpType::More => self.write(">"),
            super::OpType::MoreEqual => self.write(">="),
            super::OpType::Multiply => self.write("*"),
            super::OpType::Not => self.write("not"),
            super::OpType::Or => self.write("or"),
            super::OpType::Plus => self.write("+"),
        }?;

        if let Some(right) = &expr.right {
            right.accept(self)
        } else {
            Ok(())
        }
    }

    fn visit_literal(&mut self, expr: &Literal) -> PrinterResult {
        match &expr.literal_type {
            super::LiteralType::True => self.write("true"),
            super::LiteralType::False => self.write("false"),
            super::LiteralType::Integer(n) => self.write(&n.to_string()),
            super::LiteralType::Float(f) => self.write(&f.to_string()),
            super::LiteralType::Identifier(id) => self.write(id),
        }
    }

    fn visit_call(&mut self, expr: &Call) -> PrinterResult {
        self.write(&expr.callee)?;
        self.write("(")?;

        for arg in &expr.arguments {
            arg.accept(self)?;
            self.write(", ")?;
        }

        Ok(())
    }

    fn visit_type(&mut self, ty: &Type) -> PrinterResult {
        match &ty.kind {
            TypeKind::U8 => self.write("u8"),
            TypeKind::U16 => self.write("u16"),
            TypeKind::U32 => self.write("u32"),
            TypeKind::U64 => self.write("u64"),
            TypeKind::I8 => self.write("i8"),
            TypeKind::I16 => self.write("i16"),
            TypeKind::I32 => self.write("i32"),
            TypeKind::I64 => self.write("i64"),
            TypeKind::Float => self.write("float"),
            TypeKind::String => self.write("string"),
            TypeKind::Bool => self.write("bool"),
            TypeKind::Identifier(id) => self.write(id),
            TypeKind::Void => self.write("<void>"), // void does not exists
        }
    }

    fn visit_assignment(&mut self, expr: &Assignment) -> Result<(), io::Error> {
        expr.left.accept(self)?;
        self.write(" = ")?;
        expr.right.accept(self)
    }
}
