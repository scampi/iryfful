use std::str::SplitWhitespace;
use super::Tokenizer;
use super::Token;
use super::filter::TokenFilter;
use super::BaseTokenizer;

pub struct WhiteSpaceTokenizer<'a> {
    base: BaseTokenizer<'a, SplitWhitespace<'a>>,
}

impl<'a> WhiteSpaceTokenizer<'a> {
    pub fn new() -> WhiteSpaceTokenizer<'a> {
        WhiteSpaceTokenizer {
            base: BaseTokenizer {
                filters: Vec::new(),
                position: 0,
                input: "".split_whitespace(),
            },
        }
    }
}

impl<'a> Tokenizer<'a> for WhiteSpaceTokenizer<'a> {
    fn add_filter(&mut self, filter: TokenFilter) {
        self.base.add_filter(filter);
    }

    fn next(&mut self) -> Option<Token> {
        self.base.next()
    }

    fn set(&mut self, input: &'a str) {
        self.base.input = input.split_whitespace();
    }
}

#[cfg(test)]
mod tests {
    use expectest::prelude::*;
    use tokenizer::Token;
    use tokenizer::filter::*;
    use super::*;

    #[test]
    fn splits_on_whitespace() {
        let data = " aaa\nbbb   ccc    ";
        let mut white_space_tokenizer = WhiteSpaceTokenizer::new();

        white_space_tokenizer.set(data);

        let next_token = white_space_tokenizer.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("aaa"),
            position: 0,
        }));

        let next_token = white_space_tokenizer.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("bbb"),
            position: 1,
        }));

        let next_token = white_space_tokenizer.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("ccc"),
            position: 2,
        }));

        let next_token = white_space_tokenizer.next();
        expect!(next_token).to(be_none());
    }

    #[test]
    fn to_lowercase_filter() {
        let data = "aaa BBB cCc";
        let mut white_space_tokenizer = WhiteSpaceTokenizer::new();

        white_space_tokenizer.set(data);

        white_space_tokenizer.add_filter(TokenFilter::LowerCase);

        let next_token = white_space_tokenizer.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("aaa"),
            position: 0,
        }));

        let next_token = white_space_tokenizer.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("bbb"),
            position: 1,
        }));

        let next_token = white_space_tokenizer.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("ccc"),
            position: 2,
        }));

        let next_token = white_space_tokenizer.next();
        expect!(next_token).to(be_none());
    }
}
