use super::{Call, FunctionStatement, LetStatement, Literal, StructStatement, Type};

#[derive(Debug, Clone)]
pub enum Definition {
    Struct(*const StructStatement),
    LocalVariable(*const LetStatement),
    Function(*const FunctionStatement),
}

impl Definition {
    pub fn is_struct(&self) -> bool {
        match self {
            Definition::Struct(_) => true,
            _ => false,
        }
    }

    pub fn is_local_variable(&self) -> bool {
        match self {
            Definition::LocalVariable(_) => true,
            _ => false,
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            Definition::Function(_) => true,
            _ => false,
        }
    }
}

pub trait Bindable {
    fn get_definition(&self) -> &Definition;
    fn set_definition(&mut self, definition: Definition);
    fn get_struct_def(&self) -> &StructStatement;
    fn get_local_variable_def(&self) -> &LetStatement;
    fn get_function_def(&self) -> &FunctionStatement;
}

macro_rules! impl_bindable {
    ($type:ty) => {
        impl Bindable for $type {
            fn get_definition(&self) -> &Definition {
                self.definition.as_ref().expect("unbound")
            }

            fn set_definition(&mut self, definition: Definition) {
                self.definition = Some(definition);
            }

            fn get_struct_def(&self) -> &StructStatement {
                if let Some(Definition::Struct(strct)) = self.definition {
                    unsafe { &(*strct) }
                } else {
                    panic!("Get struct def but was {:?}", self);
                }
            }

            fn get_local_variable_def(&self) -> &LetStatement {
                if let Some(Definition::LocalVariable(var)) = self.definition {
                    unsafe { &(*var) }
                } else {
                    panic!("Get var def but was {:?}", self);
                }
            }

            fn get_function_def(&self) -> &FunctionStatement {
                if let Some(Definition::Function(func)) = self.definition {
                    unsafe { &(*func) }
                } else {
                    panic!("Get fuc def but was {:?}", self);
                }
            }
        }
    };
}

impl_bindable!(Literal);
impl_bindable!(Call);
impl_bindable!(Type);
