use crate::ast::{
    ForStatement, FunctionStatement, GlobalStatement, Statement, StatementKind, Statements,
    WhileStatement,
};

fn build_while(for_stmt: ForStatement) -> Vec<Statement> {
    let ForStatement {
        init_decl,
        continue_expression,
        modify_expression,
        body: mut for_body,
        location,
    } = for_stmt;

    for_body.statements.push(Statement {
        kind: StatementKind::Expression {
            expr: modify_expression,
            naked: false,
        },
        location: location.clone(),
    });

    let while_stmt = WhileStatement {
        condition: continue_expression,
        body: for_body,
        location: location.clone(),
    };

    vec![
        Statement {
            kind: StatementKind::Let(init_decl),
            location: location.clone(),
        },
        Statement {
            kind: StatementKind::While(while_stmt),
            location,
        },
    ]
}

fn desugar_function_body(fn_statements: Statements) -> Statements {
    let mut desugared_stmts = Vec::new();
    let location = fn_statements.location.clone();

    for stmt in fn_statements.statements.into_iter() {
        let Statement { kind, location } = stmt;

        match kind {
            StatementKind::For(for_stmt) => {
                desugared_stmts.append(&mut build_while(for_stmt));
            }
            _ => desugared_stmts.push(Statement { kind, location }),
        }
    }

    Statements {
        statements: desugared_stmts,
        location,
    }
}

pub fn desugar_for(global_stmts: Vec<GlobalStatement>) -> Vec<GlobalStatement> {
    let mut desugared_stmts = Vec::new();
    for stmt in global_stmts.into_iter() {
        match stmt {
            // Extern functions has no so statements
            GlobalStatement::Function(fn_stmt) if !fn_stmt.is_extern => {
                let FunctionStatement {
                    name,
                    parameters,
                    return_type,
                    is_extern,
                    body,
                    location,
                    ty,
                } = fn_stmt;
                let desugared_body = desugar_function_body(body.expect("unreachable"));

                desugared_stmts.push(GlobalStatement::Function(FunctionStatement {
                    name,
                    parameters,
                    return_type,
                    is_extern,
                    body: Some(desugared_body),
                    location,
                    ty,
                }));
            }
            _ => desugared_stmts.push(stmt),
        }
    }

    desugared_stmts
}
