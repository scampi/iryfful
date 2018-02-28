use std::str::SplitWhitespace;
use super::Tokenizer;

pub struct WhiteSpaceTokenizer;

impl WhiteSpaceTokenizer {
    pub fn new(input_field: &str) -> Tokenizer<SplitWhitespace> {
        Tokenizer::new(input_field.split_whitespace())
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
        let mut white_space_tokenizer = WhiteSpaceTokenizer::new(data);

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
        let mut white_space_tokenizer = WhiteSpaceTokenizer::new(data);

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
