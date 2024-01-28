use super::{
    AddrOf, ArrayInitializer, Assignment, BinaryOperation, BreakStatement, Call, ContinueStatement,
    Expression, ForStatement, FunctionStatement, GlobalStatement, IfStatement, LetStatement,
    Literal, ReturnStatement, Statement, StatementKind, Statements, StructStatement, Type,
    TypeKind, WhileStatement,
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
        if let Some(body) = stmt.body.as_ref() {
            self.visit_statements(body)?;
        }

        Ok(())
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

        self.visit_expression(stmt.init_exp.as_ref().expect("Let has no init type!"))?;
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
        if let Some(ref exp) = stmt.exp {
            self.visit_expression(exp)?;
        }

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
            Expression::ArrayInitializer(aa) => self.visit_array_initializer(aa),
            Expression::AddrOf(addrof) => self.visit_addrof(addrof),
            Expression::Deref(_) => todo!(),
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

    fn visit_array_initializer(&mut self, expr: &'ast ArrayInitializer) -> Result<(), E> {
        for exp in &expr.values {
            self.visit_expression(exp.as_ref())?;
        }

        Ok(())
    }

    fn visit_addrof(&mut self, expr: &'ast AddrOf) -> Result<(), E> {
        self.visit_expression(&expr.expr)
    }
}

pub trait MutableVisitor<'ast, E: std::error::Error> {
    fn visit_global_statement(&mut self, stmt: &'ast mut GlobalStatement) -> Result<(), E> {
        match stmt {
            GlobalStatement::Function(f) => self.visit_function(f),
            GlobalStatement::Struct(s) => self.visit_struct(s),
            GlobalStatement::Let(l) => self.visit_let(l),
        }
    }

    fn visit_statement_kind(&mut self, stmt: &'ast mut StatementKind) -> Result<(), E> {
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

    fn visit_statements_vec(&mut self, stmts: &'ast mut [Statement]) -> Result<(), E> {
        for stmt in stmts {
            self.visit_statement_kind(&mut stmt.kind)?;
        }

        Ok(())
    }

    fn visit_function(&mut self, stmt: &'ast mut FunctionStatement) -> Result<(), E> {
        if let Some(body) = stmt.body.as_mut() {
            self.visit_statements(body)?;
        }

        Ok(())
    }

    fn visit_struct(&mut self, stmt: &'ast mut StructStatement) -> Result<(), E> {
        for (kind, _) in &mut stmt.fields {
            self.visit_type_kind(kind)?;
        }
        Ok(())
    }

    fn visit_let(&mut self, stmt: &'ast mut LetStatement) -> Result<(), E> {
        if let Some(dec_ty) = &mut stmt.declaration_type {
            self.visit_type_kind(dec_ty)?;
        }

        self.visit_expression(stmt.init_exp.as_mut().expect("Let has no init type!"))?;

        Ok(())
    }

    fn visit_statements(&mut self, stmts: &'ast mut Statements) -> Result<(), E> {
        for stmt in &mut stmts.statements {
            self.visit_statement_kind(&mut stmt.kind)?;
        }
        Ok(())
    }

    fn visit_if(&mut self, stmt: &'ast mut IfStatement) -> Result<(), E> {
        self.visit_expression(&mut stmt.condition)?;
        self.visit_statements_vec(&mut stmt.then_clause.statements)?;

        if let Some(else_clause) = &mut stmt.else_clause {
            self.visit_statements_vec(&mut else_clause.statements)?;
        }

        Ok(())
    }

    fn visit_while(&mut self, stmt: &'ast mut WhileStatement) -> Result<(), E> {
        self.visit_expression(&mut stmt.condition)?;

        self.visit_statements_vec(&mut stmt.body.statements)?;

        Ok(())
    }

    fn visit_for(&mut self, stmt: &'ast mut ForStatement) -> Result<(), E> {
        self.visit_let(&mut stmt.init_decl)?;

        self.visit_expression(&mut stmt.modify_expression)?;
        self.visit_expression(&mut stmt.continue_expression)?;

        self.visit_statements_vec(&mut stmt.body.statements)?;

        Ok(())
    }

    fn visit_return(&mut self, stmt: &'ast mut ReturnStatement) -> Result<(), E> {
        if let Some(ref mut exp) = stmt.exp {
            self.visit_expression(exp)?;
        }

        Ok(())
    }

    fn visit_break(&mut self, _: &'ast mut BreakStatement) -> Result<(), E> {
        Ok(())
    }
    fn visit_continue(&mut self, _: &'ast mut ContinueStatement) -> Result<(), E> {
        Ok(())
    }

    fn visit_expression(&mut self, expr: &'ast mut Expression) -> Result<(), E> {
        match expr {
            Expression::Group(ref mut g) => self.visit_expression(g),
            Expression::BinaryOperation(ref mut bo) => self.visit_binary_operation(bo),
            Expression::Literal(ref mut l) => self.visit_literal(l),
            Expression::Call(ref mut c) => self.visit_call(c),
            Expression::Assignment(ref mut a) => self.visit_assignment(a),
            Expression::ArrayInitializer(aa) => self.visit_array_initializer(aa),
            Expression::AddrOf(addrof) => self.visit_addrof(addrof),
            Expression::Deref(_) => todo!(),
        }
    }

    fn visit_binary_operation(&mut self, expr: &'ast mut BinaryOperation) -> Result<(), E> {
        self.visit_expression(&mut expr.left)?;
        if let Some(e) = &mut expr.right {
            self.visit_expression(e)?;
        }

        Ok(())
    }

    fn visit_literal(&mut self, expr: &'ast mut Literal) -> Result<(), E> {
        match &mut expr.literal_type {
            super::LiteralType::ArrayAccess(aa) => {
                self.visit_expression(&mut aa.identifier)?;
                self.visit_expression(&mut aa.index)
            }
            _ => Ok(()),
        }
    }

    fn visit_call(&mut self, expr: &'ast mut Call) -> Result<(), E> {
        for expr in &mut expr.arguments {
            self.visit_expression(expr)?;
        }

        Ok(())
    }

    fn visit_type(&mut self, _: &'ast mut Type) -> Result<(), E> {
        Ok(())
    }

    fn visit_type_kind(&mut self, _: &'ast mut TypeKind) -> Result<(), E> {
        Ok(())
    }

    fn visit_assignment(&mut self, expr: &'ast mut Assignment) -> Result<(), E> {
        self.visit_expression(&mut expr.left)?;
        self.visit_expression(&mut expr.right)
    }

    fn visit_array_initializer(&mut self, expr: &'ast mut ArrayInitializer) -> Result<(), E> {
        for exp in expr.values.iter_mut() {
            self.visit_expression(exp.as_mut())?;
        }

        Ok(())
    }

    fn visit_addrof(&mut self, expr: &'ast mut AddrOf) -> Result<(), E> {
        self.visit_expression(&mut expr.expr)
    }
}
