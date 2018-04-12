//! Splits a string into a list of tokens.
//!
//! Available tokenizers:
//! - [`whitespace_tokenizer::WhiteSpaceTokenizer`]: splits on whitespace
pub mod filter;
pub mod whitespace_tokenizer;

use tokenizer::filter::Filter;

/// `Token` is a type that holds an owned slice of the input string after being split by the tokenizer.
#[derive(Debug, PartialEq)]
pub struct Token {
    /// The position of the token in the input string
    pub position: u32,
    /// A split outputted by a tokenizer
    pub token: String,
}

/// An interface for splitting an input string and further applying [filter::Filter]s on each
/// split.
pub trait Tokenizer {
    /// Returns a list of [`filter::TokenFilter`] to be applied on each split.
    fn get_filters(&self) -> &Vec<filter::TokenFilter>;

    /// Returns an [`Iterator`] over the splits outputted by the tokenizer for the given string.
    fn splits<'a>(&self, input: &'a str) -> Box<Iterator<Item = &'a str> + 'a>;

    /// Adds a [`filter::TokenFilter`].
    ///
    /// The order of the filters is important for the final resulting [`Token`].
    fn add_filter(&mut self, filter: filter::TokenFilter);

    /// Returns an [`Iterator`] over the [`Token`]s created from the outputted slices of
    /// [`Tokenizer::splits`].
    ///
    /// Each token is processed with the configured list of [`filter::TokenFilter`]s.
    ///
    /// # Examples
    ///
    /// ```
    /// use ::iryfful::tokenizer::Tokenizer;
    /// use ::iryfful::tokenizer::Token;
    /// use ::iryfful::tokenizer::whitespace_tokenizer::WhiteSpaceTokenizer;
    ///
    /// let tok = WhiteSpaceTokenizer::new();
    ///
    /// let mut iter = tok.tokenize("aaa bbb \n\n\tccc");
    ///
    /// assert_eq!(iter.next(), Some(Token{ position: 1, token: String::from("aaa") }));
    /// assert_eq!(iter.next(), Some(Token{ position: 2, token: String::from("bbb") }));
    /// assert_eq!(iter.next(), Some(Token{ position: 3, token: String::from("ccc") }));
    /// assert_eq!(iter.next(), None);
    /// ```
    fn tokenize<'a>(&'a self, input: &'a str) -> Box<Iterator<Item = Token> + 'a> {
        // start the position at 1 to ease out of bounds positions
        let mut pos = 1;
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
