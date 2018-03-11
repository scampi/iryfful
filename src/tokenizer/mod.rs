pub mod filter;
pub mod white_space_tokenizer;

use tokenizer::filter::Filter;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub position: u32,
    pub token: String,
}

pub trait Tokenizer<'a>
where
    <Self as Tokenizer<'a>>::Iter: Iterator<Item = &'a str>,
{
    type Iter;

    fn add_filter(&mut self, filter: filter::TokenFilter);

    fn tokenize(&self, input: &'a str) -> InputIterator<'a, Self::Iter>;
}

pub struct InputIterator<'a, T: 'a>
where
    T: Iterator<Item = &'a str>,
{
    position: u32,
    iter: T,
    filters: Vec<filter::TokenFilter>,
}

impl<'a, T> Iterator for InputIterator<'a, T>
where
    T: Iterator<Item = &'a str>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        match self.iter.next() {
            Some(part) => {
                let mut token = Token {
                    token: String::from(part),
                    position: self.position,
                };
                for filter in self.filters.iter() {
                    filter.apply(&mut token);
                }
                self.position += 1;
                Some(token)
            }
            None => None,
        }
    }
}
