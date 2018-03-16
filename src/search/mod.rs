use index::Index;

pub mod query;

pub struct IndexSearcher<'a> {
    index: Index<'a>,
}

impl<'a> IndexSearcher<'a> {
    pub fn new(index: Index<'a>) -> IndexSearcher<'a> {
        IndexSearcher { index }
    }

    pub fn search<'q, T>(&'a self, query: &'q T) -> Box<Iterator<Item = u32> + 'a>
    where
        T: query::Query<'q, 'a> + 'q,
    {
        Box::new(query.execute(&self.index))
    }
}
