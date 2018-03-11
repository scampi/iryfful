use std::str::SplitWhitespace;
use super::Tokenizer;
use super::Token;
use super::InputIterator;
use super::filter::TokenFilter;
use super::filter::Filter;

pub struct WhiteSpaceTokenizer {
    filters: Vec<TokenFilter>,
}

impl<'a> Tokenizer<'a> for WhiteSpaceTokenizer {
    type Iter = SplitWhitespace<'a>;

    fn add_filter(&mut self, filter: TokenFilter) {
        self.filters.push(filter);
    }

    fn tokenize(&'a self, input: &'a str) -> InputIterator<'a, Self::Iter> {
        InputIterator {
            position: 0,
            iter: input.split_whitespace(),
            //apply_filters: Box::new(|token: &mut Token| {
            //for filter in &self.filters.iter() {
            //filter.apply(token);
            //}
            //}),
        }
    }
}

impl WhiteSpaceTokenizer {
    fn new() -> WhiteSpaceTokenizer {
        WhiteSpaceTokenizer {
            filters: Vec::new(),
        }
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
        let white_space_tokenizer = WhiteSpaceTokenizer::new();

        let mut iter = white_space_tokenizer.tokenize(" aaa\nbbb   ccc    ");

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("aaa"),
            position: 0,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("bbb"),
            position: 1,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("ccc"),
            position: 2,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_none());
    }

    #[test]
    fn reuse_tokenizer() {
        let white_space_tokenizer = WhiteSpaceTokenizer::new();

        let mut iter = white_space_tokenizer.tokenize("aaa bbb");

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("aaa"),
            position: 0,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("bbb"),
            position: 1,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_none());

        let mut iter = white_space_tokenizer.tokenize("ccc ddd");

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("ccc"),
            position: 0,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("ddd"),
            position: 1,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_none());
    }

    //#[test]
    //fn to_lowercase_filter() {
    //let data = String::from("aaa BBB cCc");
    //let mut white_space_tokenizer = WhiteSpaceTokenizer::new();

    //white_space_tokenizer.set(data);

    //white_space_tokenizer.add_filter(TokenFilter::LowerCase);

    //let next_token = white_space_tokenizer.next();
    //expect!(next_token).to(be_some().value(Token {
    //token: String::from("aaa"),
    //position: 0,
    //}));

    //let next_token = white_space_tokenizer.next();
    //expect!(next_token).to(be_some().value(Token {
    //token: String::from("bbb"),
    //position: 1,
    //}));

    //let next_token = white_space_tokenizer.next();
    //expect!(next_token).to(be_some().value(Token {
    //token: String::from("ccc"),
    //position: 2,
    //}));

    //let next_token = white_space_tokenizer.next();
    //expect!(next_token).to(be_none());
    //}
}
