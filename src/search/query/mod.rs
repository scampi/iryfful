use super::IndexSearcher;
use super::SearchHit;

pub mod boolean_query;
pub mod phrase_query;
pub mod term_query;

pub trait Query {
    fn execute<'q, 'i: 'q>(
        &'q self,
        index_search: &'i IndexSearcher,
    ) -> Box<Iterator<Item = SearchHit> + 'q>;
}
