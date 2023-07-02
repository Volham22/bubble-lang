use super::{
    Assignment, BinaryOperation, BreakStatement, Call, ContinueStatement, Expression, ForStatement,
    FunctionStatement, GlobalStatement, IfStatement, LetStatement, Literal, ReturnStatement,
    Statement, StatementKind, Statements, StructStatement, Type, TypeKind, WhileStatement,
};

/// Default AST visitor
/// E: An error the visitor may encounter
pub trait Visitor<'ast, E: std::error::Error> {
    fn visit_statements_vec(&mut self, stmts: &'ast [Statement]) -> Result<(), E> {
        for stmt in stmts {
            self.visit_statement_kind(&stmt.kind)?;
        }

        Ok(())
    }

    fn visit_statement_kind(&mut self, stmt: &'ast StatementKind) -> Result<(), E> {
        match stmt {
            super::StatementKind::If(s) => self.visit_if(s),
            super::StatementKind::Let(s) => self.visit_let(s),
            super::StatementKind::While(s) => self.visit_while(s),
            super::StatementKind::For(s) => self.visit_for(s),
            super::StatementKind::Return(s) => self.visit_return(s),
            super::StatementKind::Break(s) => self.visit_break(s),
            super::StatementKind::Continue(s) => self.visit_continue(s),
            super::StatementKind::Expression { expr, .. } => self.visit_expression(expr),
        }
    }

    fn visit_global_statement(&mut self, stmt: &'ast GlobalStatement) -> Result<(), E> {
        match stmt {
            GlobalStatement::Function(f) => self.visit_function(f),
            GlobalStatement::Struct(s) => self.visit_struct(s),
            GlobalStatement::Let(l) => self.visit_let(l),
        }
    }

    fn visit_function(&mut self, stmt: &'ast FunctionStatement) -> Result<(), E> {
        self.visit_statements_vec(&stmt.body.statements)
    }

    fn visit_struct(&mut self, stmt: &'ast StructStatement) -> Result<(), E> {
        for (kind, _) in &stmt.fields {
            self.visit_type_kind(kind)?;
        }

        Ok(())
    }

    fn visit_let(&mut self, stmt: &'ast LetStatement) -> Result<(), E> {
        if let Some(dec_ty) = &stmt.declaration_type {
            self.visit_type_kind(dec_ty)?;
        }

        self.visit_expression(&stmt.init_exp)?;
        Ok(())
    }

    fn visit_statements(&mut self, stmts: &'ast Statements) -> Result<(), E> {
        self.visit_statements_vec(&stmts.statements)
    }

    fn visit_if(&mut self, stmt: &'ast IfStatement) -> Result<(), E> {
        self.visit_expression(&stmt.condition)?;
        self.visit_statements_vec(&stmt.then_clause.statements)?;

        if let Some(else_clause) = &stmt.else_clause {
            self.visit_statements_vec(&else_clause.statements)?;
        }

        Ok(())
    }

    fn visit_while(&mut self, stmt: &'ast WhileStatement) -> Result<(), E> {
        self.visit_expression(&stmt.condition)?;
        self.visit_statements_vec(&stmt.body.statements)
    }

    fn visit_for(&mut self, stmt: &'ast ForStatement) -> Result<(), E> {
        self.visit_let(&stmt.init_decl)?;

        self.visit_expression(&stmt.modify_expression)?;
        self.visit_expression(&stmt.continue_expression)?;

        self.visit_statements_vec(&stmt.body.statements)?;

        Ok(())
    }

    fn visit_return(&mut self, stmt: &'ast ReturnStatement) -> Result<(), E> {
        self.visit_expression(&stmt.exp)?;
        Ok(())
    }

    fn visit_break(&mut self, _: &'ast BreakStatement) -> Result<(), E> {
        Ok(())
    }
    fn visit_continue(&mut self, _: &'ast ContinueStatement) -> Result<(), E> {
        Ok(())
    }

    fn visit_expression(&mut self, expr: &'ast Expression) -> Result<(), E> {
        match expr {
            Expression::Group(g) => self.visit_expression(g),
            Expression::BinaryOperation(bo) => self.visit_binary_operation(bo),
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Call(c) => self.visit_call(c),
            Expression::Assignment(a) => self.visit_assignment(a),
        }
    }

    fn visit_binary_operation(&mut self, expr: &'ast BinaryOperation) -> Result<(), E> {
        self.visit_expression(&expr.left)?;
        if let Some(ref e) = &expr.right {
            self.visit_expression(e)?;
        }

        Ok(())
    }

    fn visit_literal(&mut self, _: &'ast Literal) -> Result<(), E> {
        Ok(())
    }

    fn visit_call(&mut self, expr: &'ast Call) -> Result<(), E> {
        for expr in &expr.arguments {
            self.visit_expression(expr)?;
        }

        Ok(())
    }

    fn visit_type(&mut self, _: &'ast Type) -> Result<(), E> {
        Ok(())
    }

    fn visit_type_kind(&mut self, _: &'ast TypeKind) -> Result<(), E> {
        Ok(())
    }

    fn visit_assignment(&mut self, expr: &'ast Assignment) -> Result<(), E> {
        self.visit_expression(&expr.left)?;
        self.visit_expression(&expr.right)
    }
}

pub trait MutableVisitor<E: std::error::Error> {
    fn visit_global_statement(&mut self, stmt: &mut GlobalStatement) -> Result<(), E> {
        match stmt {
            GlobalStatement::Function(f) => f.accept_mut(self),
            GlobalStatement::Struct(s) => s.accept_mut(self),
            GlobalStatement::Let(l) => l.accept_mut(self),
        }
    }

    fn visit_function(&mut self, stmt: &mut FunctionStatement) -> Result<(), E> {
        for stmt in &mut stmt.body.statements {
            stmt.kind.accept_mut(self)?;
        }

        Ok(())
    }

    fn visit_struct(&mut self, stmt: &mut StructStatement) -> Result<(), E> {
        for (kind, _) in &mut stmt.fields {
            kind.accept_mut(self)?;
        }
        Ok(())
    }

    fn visit_let(&mut self, stmt: &mut LetStatement) -> Result<(), E> {
        if let Some(dec_ty) = &mut stmt.declaration_type {
            dec_ty.accept_mut(self)?;
        }

        stmt.init_exp.accept_mut(self)?;
        Ok(())
    }

    fn visit_statements(&mut self, stmts: &mut Statements) -> Result<(), E> {
        for stmt in &mut stmts.statements {
            stmt.kind.accept_mut(self)?;
        }
        Ok(())
    }

    fn visit_if(&mut self, stmt: &mut IfStatement) -> Result<(), E> {
        stmt.condition.accept_mut(self)?;

        for then_stmt in &mut stmt.then_clause.statements {
            then_stmt.kind.accept_mut(self)?;
        }

        if let Some(else_clause) = &mut stmt.else_clause {
            for else_stmt in &mut else_clause.statements {
                else_stmt.kind.accept_mut(self)?;
            }
        }
        Ok(())
    }

    fn visit_while(&mut self, stmt: &mut WhileStatement) -> Result<(), E> {
        stmt.condition.accept_mut(self)?;

        for while_stmt in &mut stmt.body.statements {
            while_stmt.kind.accept_mut(self)?;
        }

        Ok(())
    }

    fn visit_for(&mut self, stmt: &mut ForStatement) -> Result<(), E> {
        stmt.init_decl.accept_mut(self)?;

        stmt.modify_expression.accept_mut(self)?;
        stmt.continue_expression.accept_mut(self)?;

        for for_stmt in &mut stmt.body.statements {
            for_stmt.kind.accept_mut(self)?;
        }

        Ok(())
    }

    fn visit_return(&mut self, stmt: &mut ReturnStatement) -> Result<(), E> {
        stmt.exp.accept_mut(self)?;
        Ok(())
    }

    fn visit_break(&mut self, _: &mut BreakStatement) -> Result<(), E> {
        Ok(())
    }
    fn visit_continue(&mut self, _: &mut ContinueStatement) -> Result<(), E> {
        Ok(())
    }

    fn visit_expression(&mut self, expr: &mut Expression) -> Result<(), E> {
        match expr {
            Expression::Group(g) => g.accept_mut(self),
            Expression::BinaryOperation(bo) => bo.accept_mut(self),
            Expression::Literal(l) => l.accept_mut(self),
            Expression::Call(c) => c.accept_mut(self),
            Expression::Assignment(a) => a.accept_mut(self),
        }
    }

    fn visit_binary_operation(&mut self, expr: &mut BinaryOperation) -> Result<(), E> {
        expr.left.accept_mut(self)?;
        if let Some(e) = &mut expr.right {
            e.accept_mut(self)?;
        }

        Ok(())
    }

    fn visit_literal(&mut self, _: &mut Literal) -> Result<(), E> {
        Ok(())
    }

    fn visit_call(&mut self, expr: &mut Call) -> Result<(), E> {
        for expr in &mut expr.arguments {
            expr.accept_mut(self)?;
        }

        Ok(())
    }

    fn visit_type(&mut self, _: &mut Type) -> Result<(), E> {
        Ok(())
    }

    fn visit_type_kind(&mut self, _: &mut TypeKind) -> Result<(), E> {
        Ok(())
    }

    fn visit_assignment(&mut self, expr: &mut Assignment) -> Result<(), E> {
        expr.left.accept_mut(self)?;
        expr.right.accept_mut(self)
    }
}
