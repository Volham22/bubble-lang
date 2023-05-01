use super::{
    BinaryOperation, BreakStatement, Call, ContinueStatement, Expression, ForStatement,
    FunctionStatement, GlobalStatement, IfStatement, LetStatement, Literal, ReturnStatement,
    Statements, StructStatement, TypeKind, WhileStatement,
};

/// Default AST visitor
/// E: An error the visitor may encounter
pub trait Visitor<E: std::error::Error> {
    fn visit_global_statement(&mut self, stmt: &GlobalStatement) -> Result<(), E> {
        match stmt {
            GlobalStatement::Function(f) => f.accept(self),
            GlobalStatement::Struct(s) => s.accept(self),
            GlobalStatement::Let(l) => l.accept(self),
        }
    }

    fn visit_function(&mut self, stmt: &FunctionStatement) -> Result<(), E> {
        for stmt in &stmt.body.statements {
            stmt.kind.accept(self)?;
        }

        Ok(())
    }

    fn visit_struct(&mut self, stmt: &StructStatement) -> Result<(), E> {
        for (kind, _) in &stmt.fields {
            kind.accept(self)?;
        }
        Ok(())
    }

    fn visit_let(&mut self, stmt: &LetStatement) -> Result<(), E> {
        if let Some(dec_ty) = &stmt.declaration_type {
            dec_ty.accept(self)?;
        }

        stmt.init_exp.accept(self)?;
        Ok(())
    }

    fn visit_statements(&mut self, stmts: &Statements) -> Result<(), E> {
        for stmt in &stmts.statements {
            stmt.kind.accept(self)?;
        }
        Ok(())
    }

    fn visit_if(&mut self, stmt: &IfStatement) -> Result<(), E> {
        stmt.condition.accept(self)?;

        for then_stmt in &stmt.then_clause.statements {
            then_stmt.kind.accept(self)?;
        }

        if let Some(else_clause) = &stmt.else_clause {
            for else_stmt in &else_clause.statements {
                else_stmt.kind.accept(self)?;
            }
        }
        Ok(())
    }

    fn visit_while(&mut self, stmt: &WhileStatement) -> Result<(), E> {
        stmt.condition.accept(self)?;

        for while_stmt in &stmt.body.statements {
            while_stmt.kind.accept(self)?;
        }
        Ok(())
    }

    fn visit_for(&mut self, stmt: &ForStatement) -> Result<(), E> {
        stmt.init_expression.accept(self)?;

        if let Some(ty) = &stmt.init_type {
            ty.kind.accept(self)?;
        }

        stmt.modify_expression.accept(self)?;
        stmt.continue_expression.accept(self)?;

        for for_stmt in &stmt.body.statements {
            for_stmt.kind.accept(self)?;
        }

        Ok(())
    }

    fn visit_return(&mut self, stmt: &ReturnStatement) -> Result<(), E> {
        stmt.exp.accept(self)?;
        Ok(())
    }

    fn visit_break(&mut self, _: &BreakStatement) -> Result<(), E> {
        Ok(())
    }
    fn visit_continue(&mut self, _: &ContinueStatement) -> Result<(), E> {
        Ok(())
    }

    fn visit_expression(&mut self, expr: &Expression) -> Result<(), E> {
        match expr {
            Expression::Group(g) => g.accept(self),
            Expression::BinaryOperation(bo) => bo.accept(self),
            Expression::Literal(l) => l.accept(self),
            Expression::Call(c) => c.accept(self),
        }
    }

    fn visit_binary_operation(&mut self, expr: &BinaryOperation) -> Result<(), E> {
        expr.left.accept(self)?;
        if let Some(e) = &expr.right {
            e.accept(self)?;
        }

        Ok(())
    }

    fn visit_literal(&mut self, _: &Literal) -> Result<(), E> {
        Ok(())
    }

    fn visit_call(&mut self, expr: &Call) -> Result<(), E> {
        for expr in &expr.arguments {
            expr.accept(self)?;
        }

        Ok(())
    }

    fn visit_type_kind(&mut self, _: &TypeKind) -> Result<(), E> {
        Ok(())
    }
}
