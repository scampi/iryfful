pub mod filter;
pub mod white_space_tokenizer;

use tokenizer::filter::Filter;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub position: u32,
    pub token: String,
}

pub trait Tokenizer<'a> {
    fn add_filter(&mut self, filter: filter::TokenFilter);

    fn next(&mut self) -> Option<Token>;

    fn set(&mut self, _: &'a str) {
        panic!("set method must be implemented");
    }
}

struct BaseTokenizer<'a, T>
where
    T: Iterator<Item = &'a str>,
{
    filters: Vec<filter::TokenFilter>,
    position: u32,
    input: T,
}

impl<'a, T> Tokenizer<'a> for BaseTokenizer<'a, T>
where
    T: Iterator<Item = &'a str>,
{
    fn add_filter(&mut self, filter: filter::TokenFilter) {
        self.filters.push(filter);
    }

    fn next(&mut self) -> Option<Token> {
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
