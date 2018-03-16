use index::Index;

pub mod term_query;

pub trait Query<'q, 'i> {
    fn execute(&'q self, index: &'i Index) -> Box<Iterator<Item = u32> + 'i>;
}
