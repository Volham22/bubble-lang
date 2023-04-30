use super::{
    BinaryOperation, BreakStatement, Call, ContinueStatement, Expression, ForStatement,
    FunctionStatement, GlobalStatement, IfStatement, LetStatement, Literal, ReturnStatement,
    Statements, StructStatement, WhileStatement,
};

/// Default AST visitor
pub trait Visitor {
    fn visit_global_statement(&self, stmt: &GlobalStatement) {
        match stmt {
            GlobalStatement::Function(f) => f.accept(self),
            GlobalStatement::Struct(s) => s.accept(self),
            GlobalStatement::Let(l) => l.accept(self),
        }
    }

    fn visit_function(&self, stmt: &FunctionStatement) {
        for stmt in &stmt.body.statements {
            stmt.kind.accept(self)
        }
    }

    fn visit_struct(&self, stmt: &StructStatement) {
        for (kind, _) in &stmt.fields {
            kind.accept(self);
        }
    }

    fn visit_let(&self, stmt: &LetStatement) {
        if let Some(dec_ty) = &stmt.declaration_type {
            dec_ty.accept(self);
        }

        stmt.init_exp.accept(self);
    }

    fn visit_statements(&self, stmts: &Statements) {
        for stmt in &stmts.statements {
            stmt.kind.accept(self);
        }
    }

    fn visit_if(&self, stmt: &IfStatement) {
        stmt.condition.accept(self);

        for then_stmt in &stmt.then_clause.statements {
            then_stmt.kind.accept(self);
        }

        if let Some(else_clause) = &stmt.else_clause {
            for else_stmt in &else_clause.statements {
                else_stmt.kind.accept(self);
            }
        }
    }

    fn visit_while(&self, stmt: &WhileStatement) {
        stmt.condition.accept(self);

        for while_stmt in &stmt.body.statements {
            while_stmt.kind.accept(self);
        }
    }

    fn visit_for(&self, stmt: &ForStatement) {
        stmt.init_expression.accept(self);

        if let Some(ty) = &stmt.init_type {
            ty.kind.accept(self);
        }

        stmt.modify_expression.accept(self);
        stmt.continue_expression.accept(self);

        for for_stmt in &stmt.body.statements {
            for_stmt.kind.accept(self);
        }
    }

    fn visit_return(&self, stmt: &ReturnStatement) {
        stmt.exp.accept(self);
    }

    fn visit_break(&self, _: &BreakStatement) {}
    fn visit_continue(&self, _: &ContinueStatement) {}

    fn visit_expression(&self, expr: &Expression) {
        match expr {
            Expression::Group(g) => g.accept(self),
            Expression::BinaryOperation(bo) => bo.accept(self),
            Expression::Literal(_) => (),
            Expression::Call(c) => c.accept(self),
        }
    }

    fn visit_binary_operation(&self, expr: &BinaryOperation) {
        expr.left.accept(self);
        if let Some(e) = &expr.right {
            e.accept(self);
        }
    }

    fn visit_literal(&self, _: &Literal) {}

    fn visit_call(&self, expr: &Call) {
        for expr in &expr.arguments {
            expr.accept(self);
        }
    }
}
