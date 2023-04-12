#[derive(Debug)]
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
