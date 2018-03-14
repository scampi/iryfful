pub mod filter;
pub mod white_space_tokenizer;

use tokenizer::filter::Filter;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub position: u32,
    pub token: String,
}

pub trait Tokenizer {
    fn get_filters(&self) -> &Vec<filter::TokenFilter>;

    fn splits<'a>(&self, input: &'a str) -> Box<Iterator<Item = &'a str> + 'a>;

    fn add_filter(&mut self, filter: filter::TokenFilter);

    fn tokenize<'a>(&'a self, input: &'a str) -> Box<Iterator<Item = Token> + 'a> {
        let mut pos = 0;
        Box::new(self.splits(input).map(move |part| {
            let mut token = Token {
                token: String::from(part),
                position: pos,
            };
            for filter in self.get_filters().iter() {
                filter.apply(&mut token);
            }
            pos += 1;
            token
        }))
    }
}
