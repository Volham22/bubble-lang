macro_rules! impl_locatable {
    ( $( $t:ty ),+ $(,)? ) => {

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

#[macro_export]
macro_rules! location_to_span {
    ( $loc:ident ) => {
        $loc.begin..$loc.end
    };
}

#[derive(Debug, Clone)]
pub struct TokenLocation {
    pub begin: usize,
    pub end: usize,
}

impl From<TokenLocation> for Range<usize> {
    fn from(value: TokenLocation) -> Self {
        value.begin..value.end
    }
}

impl TokenLocation {
    pub fn new(begin: usize, end: usize) -> Self {
        Self { begin, end }
    }
}

pub trait Locatable {
    fn get_location(&self) -> &TokenLocation;
}

use std::ops::Range;

pub(crate) use impl_locatable;
pub use location_to_span;
