use super::Query;
use super::SearchHit;
use search::IndexSearcher;

#[derive(Default)]
pub struct BooleanQuery<'bq> {
    must: Vec<Box<Query + 'bq>>,
}

impl<'bq> BooleanQuery<'bq> {
    pub fn must<T>(&mut self, query: T)
    where
        T: Query + 'bq,
    {
        self.must.push(Box::new(query));
    }
}

impl<'bq> Query for BooleanQuery<'bq> {
    fn execute<'q, 'i: 'q>(
        &'q self,
        index_search: &'i IndexSearcher,
    ) -> Box<Iterator<Item = SearchHit> + 'q> {
        let must_results = self.must
            .iter()
            .map(|query| Box::new(query.execute(index_search)))
            .collect();

        Box::new(
            index_search
                .step_on_matching_doc(must_results)
                .map(|(doc_id, _)| SearchHit::new(doc_id)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use index::Index;
    use index::document::Document;
    use search::IndexSearcher;
    use search::SearchHit;
    use search::query::term_query::TermQuery;
    use tokenizer::whitespace_tokenizer::WhiteSpaceTokenizer;

    #[test]
    fn test_must_term_queries() {
        let mut index = Index::new();
        index.set_mapping(String::from("field1"), WhiteSpaceTokenizer::new());

        let mut doc = Document::new();
        doc.add_field("field1", "aaa ccc");
        index.add_doc(doc);

        let mut doc = Document::new();
        doc.add_field("field1", "aaa bbb");
        index.add_doc(doc);

        let mut doc = Document::new();
        doc.add_field("field1", "aaa bbb ccc");
        index.add_doc(doc);

        let index_search = IndexSearcher::new(&index);

        let mut bq: BooleanQuery = Default::default();
        bq.must(TermQuery::new("field1", "aaa"));
        bq.must(TermQuery::new("field1", "ccc"));

        let mut iter = bq.execute(&index_search);

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(0)));

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(2)));

        let next_doc = iter.next();
        assert_eq!(next_doc, None);
    }
}
