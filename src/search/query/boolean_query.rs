//! Match a document that fulfils a boolean combination of queries
//!
//! The `must` clause defines queries that must match a document. It is added thanks to
//! [`BooleanQuery::must`] method.
//!
//! # Examples
//!
//! ```no_run
//! use ::iryfful::search::query::boolean_query::BooleanQuery;
//! use ::iryfful::search::query::phrase_query::PhraseQuery;
//!
//! // This can match a document with "aaa bbb eee ccc ddd" but not "aaa bbb ccc eee ddd" because
//! // the second query is not fulfilled.
//! let mut bq: BooleanQuery = Default::default();
//! bq.must(PhraseQuery::new("field1", vec!["aaa", "bbb"]));
//! bq.must(PhraseQuery::new("field1", vec!["ccc", "ddd"]));
//! ```
use super::Query;
use super::SearchHit;
use index::posting_lists::DocItem;
use search::DocIterator;
use search::IndexSearcher;

#[derive(Debug, Default)]
pub struct BooleanQuery<'bq> {
    must: Vec<Box<Query + 'bq>>,
    must_not: Vec<Box<Query + 'bq>>,
}

impl<'bq> BooleanQuery<'bq> {
    /// Adds a query that must be matched
    pub fn must<T>(&mut self, query: T)
    where
        T: Query + 'bq,
    {
        self.must.push(Box::new(query));
    }

    /// Adds a query that must not be matched
    pub fn must_not<T>(&mut self, query: T)
    where
        T: Query + 'bq,
    {
        self.must_not.push(Box::new(query));
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
        let mut must_not_results = index_search.disjunction(
            self.must_not
                .iter()
                .map(move |query| Box::new(query.execute(index_search)))
                .collect(),
        );

        let mut current_must_not_doc = match must_not_results.next() {
            None => None,
            Some(item) => Some(item.get_doc_id()),
        };
        Box::new(
            index_search
                .conjunction(must_results)
                .filter_map(move |(doc_id, _)| {
                    match current_must_not_doc {
                        // the current doc in the must_not clause is a match, let't remove it
                        Some(current_must_not_doc_id) if current_must_not_doc_id == doc_id => None,
                        Some(current_must_not_doc_id) if current_must_not_doc_id < doc_id => {
                            match must_not_results.advance(doc_id) {
                                // no doc in the must_not clause, keep all the doc
                                None => {
                                    current_must_not_doc = None;
                                    Some(SearchHit::new(doc_id))
                                }
                                // the doc_id is a match in the must_not clause, let's remove it
                                Some((true, next_item)) => {
                                    current_must_not_doc = Some(next_item.get_doc_id());
                                    None
                                }
                                // the doc_id is not a match in the must_not clause, keep it
                                Some((false, next_item)) => {
                                    current_must_not_doc = Some(next_item.get_doc_id());
                                    Some(SearchHit::new(doc_id))
                                }
                            }
                        }
                        // keep all the doc because either there is no doc in the must_not clause,
                        // or doc ID from the must clause is lower than the current doc ID of the
                        // must_not clause
                        _ => Some(SearchHit::new(doc_id)),
                    }
                }),
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

    #[test]
    fn test_must_not1() {
        let mut index: Index = Default::default();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc: Document = Default::default();
        doc.add_field("field1", "aaa bbb");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "bbb aaa ccc");
        index.add_doc(&doc).unwrap();

        let index_search = IndexSearcher::new(&index);

        let mut bq: BooleanQuery = Default::default();
        bq.must(TermQuery::new("field1", "aaa"));
        bq.must_not(TermQuery::new("field1", "bbb"));

        let mut iter = bq.execute(&index_search);

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(1)));

        let next_doc = iter.next();
        assert_eq!(next_doc, None);
    }

    #[test]
    fn test_must_not2() {
        let mut index: Index = Default::default();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc: Document = Default::default();
        doc.add_field("field1", "aaa bbb");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa ccc ddd");
        index.add_doc(&doc).unwrap();

        let index_search = IndexSearcher::new(&index);

        let mut bq: BooleanQuery = Default::default();
        bq.must(TermQuery::new("field1", "aaa"));
        bq.must_not(PhraseQuery::new("field1", vec!["ccc", "ddd"]));
        bq.must_not(TermQuery::new("field1", "bbb"));

        let mut iter = bq.execute(&index_search);

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(1)));

        let next_doc = iter.next();
        assert_eq!(next_doc, None);
    }

    #[test]
    fn test_must_not_overlapping_results() {
        let mut index: Index = Default::default();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc: Document = Default::default();
        doc.add_field("field1", "aaa bbb");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa bbb ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "ddd aaa");
        index.add_doc(&doc).unwrap();

        let index_search = IndexSearcher::new(&index);

        let mut bq: BooleanQuery = Default::default();
        bq.must(TermQuery::new("field1", "aaa"));
        bq.must_not(PhraseQuery::new("field1", vec!["bbb", "ccc"]));
        bq.must_not(TermQuery::new("field1", "bbb"));

        let mut iter = bq.execute(&index_search);

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(1)));

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(3)));

        let next_doc = iter.next();
        assert_eq!(next_doc, None);
    }
    #[test]
    fn test_nested_must_not() {
        let mut index: Index = Default::default();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc: Document = Default::default();
        doc.add_field("field1", "aaa bbb");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa bbb ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa ddd");
        index.add_doc(&doc).unwrap();

        let index_search = IndexSearcher::new(&index);

        let mut bq1: BooleanQuery = Default::default();
        bq1.must(TermQuery::new("field1", "bbb"));
        bq1.must(TermQuery::new("field1", "ccc"));

        let mut bq: BooleanQuery = Default::default();
        bq.must(TermQuery::new("field1", "aaa"));
        bq.must_not(bq1);

        let mut iter = bq.execute(&index_search);

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(0)));

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(1)));

        let next_doc = iter.next();
        assert_eq!(next_doc, Some(SearchHit::new(3)));

        let next_doc = iter.next();
        assert_eq!(next_doc, None);
    }
}
