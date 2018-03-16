use super::Query;
use index::Index;

pub struct TermQuery<'a> {
    field: &'a str,
    term: &'a str,
}

impl<'a> TermQuery<'a> {
    pub fn new(field: &'a str, term: &'a str) -> TermQuery<'a> {
        TermQuery { field, term }
    }
}

impl<'q, 'i> Query<'q, 'i> for TermQuery<'q> {
    fn execute(&'q self, index: &'i Index) -> Box<Iterator<Item = u32> + 'i> {
        Box::new(
            index
                .get_postings_list(&format!("{}:{}", self.field, self.term))
                .unwrap()
                .iter_docs(),
        )
    }
}

#[cfg(test)]
mod tests {
    use tokenizer::whitespace_tokenizer::WhiteSpaceTokenizer;
    use expectest::prelude::*;
    use super::*;
    use index::document::Document;

    #[test]
    fn test_hits() {
        let mut index = Index::new();
        index.set_mapping(String::from("field1"), WhiteSpaceTokenizer::new());

        let mut doc = Document::new();
        doc.add_field("field1", "aaa bbb aaa");
        index.add_doc(doc);

        let mut doc = Document::new();
        doc.add_field("field1", "bbb");
        index.add_doc(doc);

        let mut doc = Document::new();
        doc.add_field("field1", "aaa");
        index.add_doc(doc);

        let tq = TermQuery::new("field1", "aaa");
        let mut iter = tq.execute(&index);

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(0));

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(2));

        let next_doc = iter.next();
        expect!(next_doc).to(be_none());
    }
}
