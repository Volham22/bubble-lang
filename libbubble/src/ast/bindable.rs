use super::{Call, FunctionStatement, LetStatement, Literal, StructStatement, Type};

#[derive(Debug, Clone)]
pub enum Definition {
    Struct(StructStatement),
    LocalVariable(LetStatement),
    Function(FunctionStatement),
}

pub trait Bindable {
    fn get_definition(&self) -> &Definition;
    fn set_definition(&mut self, definition: Definition);
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
        }
    };
}

impl_bindable!(Literal);
impl_bindable!(Call);
impl_bindable!(Type);
