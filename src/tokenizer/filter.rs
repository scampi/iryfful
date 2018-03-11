use super::Token;

pub trait Filter {
    fn apply(&self, token: &mut Token);
}

#[derive(Clone)]
pub enum TokenFilter {
    LowerCase,
}

impl Filter for TokenFilter {
    fn apply(&self, token: &mut Token) {
        match *self {
            TokenFilter::LowerCase => token.token = token.token.to_lowercase(),
        }
    }
}
