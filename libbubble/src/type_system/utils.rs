use std::collections::HashMap;

pub type Scope<T> = HashMap<String, T>;
pub struct ScopedMap<T>(Vec<Scope<T>>);

impl<T> Default for ScopedMap<T> {
    /// Creates a default ScopedMap.
    /// Note: Every ScopedMap has a default scope
    fn default() -> Self {
        Self(vec![Scope::new()])
    }
}

impl<T> ScopedMap<T> {
    pub fn new_scope(&mut self) {
        self.0.push(Scope::new());
    }

    pub fn delete_scope(&mut self) {
        assert!(!self.0.is_empty());
        self.0.pop().unwrap();
    }

    pub fn insert_symbol(&mut self, name: &str, element: T) {
        self.0
            .last_mut()
            .expect("insert on empty scoped map!")
            .insert(name.to_string(), element);
    }

    pub fn find_symbol(&self, symbol: &str) -> Option<&T> {
        self.0.iter().rev().find_map(|scope| scope.get(symbol))
    }

    #[cfg(test)] // it is only used to tests the implementation
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::ScopedMap;
    use crate::ast;

    #[test]
    fn default_scoped_map() {
        let def: ScopedMap<ast::LetStatement> = ScopedMap::default();
        assert_eq!(def.len(), 1);
    }

    #[test]
    fn scope_map_find_one() {
        let mut def: ScopedMap<ast::LetStatement> = ScopedMap::default();
        def.insert_symbol(
            "a",
            ast::LetStatement::new(
                0,
                0,
                "a".to_string(),
                None,
                Some(Box::new(ast::Expression::Literal(ast::Literal::new(
                    0,
                    0,
                    ast::LiteralType::True,
                )))),
            ),
        );
        let a = def.find_symbol("a");
        assert!(a.is_some());
    }

    #[test]
    fn scope_map_find_multiple_insert() {
        let mut def: ScopedMap<ast::LetStatement> = ScopedMap::default();
        def.insert_symbol(
            "a",
            ast::LetStatement::new(
                0,
                0,
                "a".to_string(),
                None,
                Some(Box::new(ast::Expression::Literal(ast::Literal::new(
                    0,
                    0,
                    ast::LiteralType::True,
                )))),
            ),
        );

        def.insert_symbol(
            "b",
            ast::LetStatement::new(
                0,
                0,
                "b".to_string(),
                None,
                Some(Box::new(ast::Expression::Literal(ast::Literal::new(
                    0,
                    0,
                    ast::LiteralType::True,
                )))),
            ),
        );

        let a = def.find_symbol("a");
        assert!(a.is_some());
        let b = def.find_symbol("a");
        assert!(b.is_some());
    }

    #[test]
    fn scoped_map_find_nearest_scope() {
        let mut def: ScopedMap<ast::LetStatement> = ScopedMap::default();
        def.insert_symbol(
            "a",
            ast::LetStatement::new(
                0,
                0,
                "far".to_string(),
                None,
                Some(Box::new(ast::Expression::Literal(ast::Literal::new(
                    0,
                    0,
                    ast::LiteralType::True,
                )))),
            ),
        );

        def.new_scope();
        assert_eq!(def.len(), 2);

        def.insert_symbol(
            "a",
            ast::LetStatement::new(
                0,
                0,
                "near".to_string(),
                None,
                Some(Box::new(ast::Expression::Literal(ast::Literal::new(
                    0,
                    0,
                    ast::LiteralType::True,
                )))),
            ),
        );

        let a = def.find_symbol("a");
        assert_eq!(a.expect("a not found!").name, "near");
    }

    #[test]
    fn scoped_map_outer_scope() {
        let mut def: ScopedMap<ast::LetStatement> = ScopedMap::default();
        def.insert_symbol(
            "a",
            ast::LetStatement::new(
                0,
                0,
                "a".to_string(),
                None,
                Some(Box::new(ast::Expression::Literal(ast::Literal::new(
                    0,
                    0,
                    ast::LiteralType::True,
                )))),
            ),
        );
        def.new_scope();
        let a = def.find_symbol("a");
        assert!(a.is_some());
    }

    #[test]
    fn scope_map_out_of_scope() {
        let mut def: ScopedMap<ast::LetStatement> = ScopedMap::default();
        def.new_scope();
        assert_eq!(def.len(), 2);
        def.insert_symbol(
            "a",
            ast::LetStatement::new(
                0,
                0,
                "a".to_string(),
                None,
                Some(Box::new(ast::Expression::Literal(ast::Literal::new(
                    0,
                    0,
                    ast::LiteralType::True,
                )))),
            ),
        );
        def.delete_scope();
        assert_eq!(def.len(), 1);

        let a = def.find_symbol("a");
        assert!(a.is_none());
    }

    #[test]
    fn scope_map_not_inserted() {
        let def: ScopedMap<ast::LetStatement> = ScopedMap::default();
        assert!(def.find_symbol("var").is_none());
    }
}
