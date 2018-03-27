use super::Tokenizer;
use super::filter::TokenFilter;

pub struct WhiteSpaceTokenizer {
    filters: Vec<TokenFilter>,
}

impl Tokenizer for WhiteSpaceTokenizer {
    fn add_filter(&mut self, filter: TokenFilter) {
        self.filters.push(filter);
    }

    fn get_filters(&self) -> &Vec<TokenFilter> {
        &self.filters
    }

    fn splits<'a>(&self, input: &'a str) -> Box<Iterator<Item = &'a str> + 'a> {
        Box::new(input.split_whitespace())
    }
}

impl WhiteSpaceTokenizer {
    pub fn new() -> WhiteSpaceTokenizer {
        WhiteSpaceTokenizer {
            filters: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;
    use tokenizer::Token;
    use tokenizer::filter::*;

    #[test]
    fn splits_on_whitespace() {
        let white_space_tokenizer = WhiteSpaceTokenizer::new();

        let mut iter = white_space_tokenizer.tokenize(" aaa\nbbb   ccc    ");

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("aaa"),
            position: 1,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("bbb"),
            position: 2,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("ccc"),
            position: 3,
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
            position: 1,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("bbb"),
            position: 2,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_none());

        let mut iter = white_space_tokenizer.tokenize("ccc ddd");

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("ccc"),
            position: 1,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("ddd"),
            position: 2,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_none());
    }

    #[test]
    fn to_lowercase_filter() {
        let mut white_space_tokenizer = WhiteSpaceTokenizer::new();

        white_space_tokenizer.add_filter(TokenFilter::LowerCase);

        let mut iter = white_space_tokenizer.tokenize("aaa BBB cCc");

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("aaa"),
            position: 1,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("bbb"),
            position: 2,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_some().value(Token {
            token: String::from("ccc"),
            position: 3,
        }));

        let next_token = iter.next();
        expect!(next_token).to(be_none());
    }
}
