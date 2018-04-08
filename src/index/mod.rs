use std::collections::HashMap;
use std::collections::hash_map::Entry;
use tokenizer::Tokenizer;

pub mod document;
pub mod error;
pub mod posting_lists;

type IndexingResult<T> = Result<T, error::IndexingError>;

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

    pub fn set_mapping<T: 'a>(&mut self, field: String, tokenizer: T) -> IndexingResult<()>
    where
        T: Tokenizer,
    {
        match self.mappings.entry(field) {
            Entry::Vacant(entry) => {
                entry.insert(Box::new(tokenizer));
                Ok(())
            }
            Entry::Occupied(entry) => Err(error::IndexingError::MappingFieldAlreadyExists {
                field: entry.key().to_string(),
            }),
        }
    }

    pub fn add_doc(&mut self, doc: document::Document) -> IndexingResult<()> {
        for field in doc.fields() {
            if !self.mappings.contains_key(field.field) {
                return Err(error::IndexingError::MissingFieldMapping {
                    field: field.field.to_string(),
                });
            }
            let tokenizer = self.mappings.get(field.field).unwrap();
            for token in tokenizer.tokenize(field.value) {
                let posting = self.postings
                    .entry(format!("{}:{}", field.field, token.token))
                    .or_insert(posting_lists::Posting::new());
                posting.add_token(self.doc_id, token.position);
            }
        }
        self.doc_id += 1;
        Ok(())
    }

    pub fn get_postings_list(&self, field: &str) -> Result<&posting_lists::Posting, String> {
        self.postings
            .get(field)
            .ok_or_else(|| format!("No postings for field={}", field))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokenizer::whitespace_tokenizer::WhiteSpaceTokenizer;

    #[test]
    fn should_fail_when_setting_two_times_a_mapping_field() {
        let mut index = Index::new();

        let set_mapping_res = index.set_mapping(String::from("field1"), WhiteSpaceTokenizer::new());
        assert_eq!(set_mapping_res.is_ok(), true);

        let set_mapping_res = index.set_mapping(String::from("field1"), WhiteSpaceTokenizer::new());
        assert_eq!(set_mapping_res.is_err(), true);
    }

    #[test]
    fn should_fail_when_adding_doc_with_missing_field_mapping() {
        let mut index = Index::new();

        let set_mapping_res = index.set_mapping(String::from("field1"), WhiteSpaceTokenizer::new());
        assert_eq!(set_mapping_res.is_ok(), true);

        let set_mapping_res = index.set_mapping(String::from("field1"), WhiteSpaceTokenizer::new());
        assert_eq!(set_mapping_res.is_err(), true);
    }

    /// Should index 2 docs over two postings list
    #[test]
    fn should_create_some_postings_list() {
        let mut index = Index::new();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc = document::Document::new();
        doc.add_field("field1", "aaa bbb aaa");
        index.add_doc(doc).unwrap();

        let mut doc = document::Document::new();
        doc.add_field("field1", "bbb");
        index.add_doc(doc).unwrap();

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
