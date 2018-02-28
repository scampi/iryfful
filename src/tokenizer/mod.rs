pub mod filter;
pub mod white_space_tokenizer;

use tokenizer::filter::Filter;

#[derive(Debug, PartialEq)]
pub struct Token {
    position: i32,
    token: String,
}

pub struct Tokenizer<'a, T>
where
    T: Iterator<Item = &'a str>,
{
    filters: Vec<filter::TokenFilter>,
    position: i32,
    input: T,
}

impl<'a, T> Tokenizer<'a, T>
where
    T: Iterator<Item = &'a str>,
{
    pub fn new(iter: T) -> Tokenizer<'a, T> {
        Tokenizer {
            filters: Vec::new(),
            position: 0,
            input: iter,
        }
    }

    pub fn add_filter(&mut self, filter: filter::TokenFilter) -> () {
        self.filters.push(filter);
    }

    pub fn next(&mut self) -> Option<Token> {
        match self.input.next() {
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
