#[derive(Debug, Clone)]
pub struct TokenLocation {
    pub line: usize,
    pub column: usize,
    pub begin: usize,
    pub end: usize,
}

impl TokenLocation {
    pub fn new(begin: usize, end: usize) -> Self {
        Self {
            line: 0,
            column: 0,
            begin,
            end,
        }
    }
}

pub trait Locatable {
    fn get_location(&self) -> &TokenLocation;
}

macro_rules! impl_locatable {
    ( $( $t:ty ),* ) => {

        $(
            impl Locatable for $t {
                fn get_location(&self) -> &TokenLocation {
                    &self.location
                }
            }

            impl Locatable for & $t {
                fn get_location(&self) -> &TokenLocation {
                    &self.location
                }
            }
        )*
    };
}

pub(crate) use impl_locatable;
