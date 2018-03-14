use std::collections::HashMap;
use tokenizer::Tokenizer;

pub mod document;
pub mod posting_lists;

pub struct Index<'a> {
    doc_id: u32,
    postings: HashMap<String, posting_lists::Posting>,
    mappings: HashMap<String, Box<Tokenizer + 'a>>,
}

impl<'a> Index<'a> {
    pub fn new() -> Index<'a> {
        Index {
            doc_id: 0,
            postings: HashMap::new(),
            mappings: HashMap::new(),
        }
    }

    pub fn set_mapping<T: 'a>(&mut self, field: String, tokenizer: T)
    where
        T: Tokenizer,
    {
        self.mappings.insert(field, Box::new(tokenizer));
    }

    pub fn add_doc(&mut self, doc: document::Document) {
        for field in doc.fields() {
            let tokenizer = self.mappings
                .get(field.field)
                .expect("field not mapped in index");
            for token in tokenizer.tokenize(field.value) {
                let posting = self.postings
                    .entry(format!("{}:{}", field.field, token.token))
                    .or_insert(posting_lists::Posting::new());
                posting.add_token(self.doc_id, token.position);
            }
        }
        self.doc_id += 1;
    }
}

#[cfg(test)]
mod tests {
    use tokenizer::white_space_tokenizer::WhiteSpaceTokenizer;
    use super::*;

    /// Should index 2 docs over two postings list
    #[test]
    fn should_create_some_postings_list() {
        let mut index = Index::new();
        index.set_mapping(String::from("field1"), WhiteSpaceTokenizer::new());

        let mut doc = document::Document::new();
        doc.add_field("field1", "aaa bbb aaa");
        index.add_doc(doc);

        let mut doc = document::Document::new();
        doc.add_field("field1", "bbb");
        index.add_doc(doc);

        assert_eq!(index.postings.len(), 2);
        for (key, posting) in index.postings.iter() {
            match key.as_ref() {
                "field1:aaa" => assert_eq!(posting.len(), 1),
                "field1:bbb" => assert_eq!(posting.len(), 2),
                _ => panic!(format!("got unexpected key={}", key)),
            }
        }
    }
}
