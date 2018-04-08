use super::Query;
use index::posting_lists::DocItem;
use search::IndexSearcher;
use search::SearchHit;

#[derive(Debug)]
pub struct TermQuery<'a> {
    field: &'a str,
    term: &'a str,
}

impl<'a> TermQuery<'a> {
    pub fn new(field: &'a str, term: &'a str) -> TermQuery<'a> {
        TermQuery { field, term }
    }
}

impl<'tq> Query for TermQuery<'tq> {
    fn execute<'q, 'i: 'q>(
        &'q self,
        index_search: &'i IndexSearcher,
    ) -> Box<Iterator<Item = SearchHit> + 'q> {
        Box::new(
            index_search
                .get_index()
                .get_postings_list(&format!("{}:{}", self.field, self.term))
                .iter_docs()
                .map(|doc| SearchHit::new(doc.get_doc_id())),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;
    use index::Index;
    use index::document::Document;
    use search::IndexSearcher;
    use search::SearchHit;
    use tokenizer::whitespace_tokenizer::WhiteSpaceTokenizer;

    #[test]
    fn test_hits() {
        let mut index = Index::new();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "aaa bbb aaa");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "bbb");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "aaa");
        index.add_doc(doc).unwrap();

        let index_search = &IndexSearcher::new(&index);

        let tq = TermQuery::new("field1", "aaa");
        let mut iter = tq.execute(index_search);

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(SearchHit::new(0)));

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(SearchHit::new(2)));

        let next_doc = iter.next();
        expect!(next_doc).to(be_none());
    }
}
