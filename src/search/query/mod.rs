use super::IndexSearcher;

pub mod term_query;
pub mod phrase_query;

pub trait Query<'q, 'i> {
    fn execute(&'q self, index_search: &'i IndexSearcher) -> Box<Iterator<Item = u32> + 'i>;
}
