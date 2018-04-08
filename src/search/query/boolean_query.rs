use super::Query;
use super::SearchHit;
use search::IndexSearcher;

#[derive(Debug, Default)]
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
    use search::query::phrase_query::PhraseQuery;
    use search::query::term_query::TermQuery;
    use tokenizer::whitespace_tokenizer::WhiteSpaceTokenizer;

    #[test]
    fn test_must_term_queries() {
        let mut index: Index = Default::default();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc: Document = Default::default();
        doc.add_field("field1", "aaa ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa bbb");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa bbb ccc");
        index.add_doc(&doc).unwrap();

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

    #[test]
    fn test_must_phrase_queries() {
        let mut index: Index = Default::default();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc: Document = Default::default();
        doc.add_field("field1", "aaa bbb");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "bbb ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa bbb ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa bbb ddd ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa bbb ddd eee ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa ddd bbb ccc");
        index.add_doc(&doc).unwrap();

        let index_search = IndexSearcher::new(&index);

        let pq1 = PhraseQuery::new("field1", vec!["aaa", "bbb"]);
        let mut pq2 = PhraseQuery::new("field1", vec!["bbb", "ccc"]);
        pq2.set_slop(2);
        let mut bq: BooleanQuery = Default::default();
        bq.must(pq1);
        bq.must(pq2);

        let mut iter = bq.execute(&index_search);

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(2)));

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(3)));

        let next_doc = iter.next();
        assert_eq!(next_doc, None);
    }

    #[test]
    fn test_must_phrase_and_term_queries() {
        let mut index: Index = Default::default();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc: Document = Default::default();
        doc.add_field("field1", "aaa bbb");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "bbb ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa bbb ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa bbb ddd ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa bbb ddd eee ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa ddd bbb ccc");
        index.add_doc(&doc).unwrap();

        let index_search = IndexSearcher::new(&index);

        let mut bq: BooleanQuery = Default::default();
        bq.must(PhraseQuery::new("field1", vec!["aaa", "bbb"]));
        bq.must(TermQuery::new("field1", "ccc"));

        let mut iter = bq.execute(&index_search);

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(2)));

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(3)));

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(4)));

        let next_doc = iter.next();
        assert_eq!(next_doc, None);
    }

    #[test]
    fn test_nested_must() {
        let mut index: Index = Default::default();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc: Document = Default::default();
        doc.add_field("field1", "aaa bbb ddd");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "ddd aaa bbb ccc eee");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "bbb ccc eee");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "eee ccc bbb aaa ddd");
        index.add_doc(&doc).unwrap();

        let index_search = IndexSearcher::new(&index);

        let mut bq1: BooleanQuery = Default::default();
        bq1.must(PhraseQuery::new("field1", vec!["aaa", "bbb"]));
        bq1.must(TermQuery::new("field1", "ddd"));

        let mut bq2: BooleanQuery = Default::default();
        bq2.must(PhraseQuery::new("field1", vec!["bbb", "ccc"]));
        bq2.must(TermQuery::new("field1", "eee"));

        let mut bq: BooleanQuery = Default::default();
        bq.must(bq1);
        bq.must(bq2);

        let mut iter = bq.execute(&index_search);

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(1)));

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(3)));

        let next_doc = iter.next();
        assert_eq!(next_doc, None);
    }
}
