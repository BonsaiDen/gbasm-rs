/// A trait for bytewise iteration through assembly sources
pub trait SourceIter {
    fn get(&self) -> u8;
    fn next(&mut self) -> u8;
    fn peek(&mut self) -> u8;
    fn is_empty(&self) -> bool;
}

