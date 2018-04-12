//! Apply some operation over a token.
use super::Token;

/// Filter interface allows to apply a mutating operation over a token
pub trait Filter {
    fn apply(&self, token: &mut Token);
}

/// Type of possible builtin [`Filter`]s.
pub enum TokenFilter {
    /// Returns a lowercased version of the token
    LowerCase,
}

impl Filter for TokenFilter {
    fn apply(&self, token: &mut Token) {
        match *self {
            TokenFilter::LowerCase => token.token = token.token.to_lowercase(),
        }
    }
}
