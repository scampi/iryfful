//! Indexing logic of documents.
//!
//! The [`Index`] type provides an API for adding documents to an index and interacting with it.
use index::posting_lists::Posting;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use tokenizer::Tokenizer;

pub mod document;
pub mod error;
pub mod posting_lists;

type IndexingResult<T> = Result<T, error::IndexingError>;

#[derive(Default)]
pub struct Index<'a> {
    doc_id: u32,
    postings: HashMap<String, posting_lists::PostingImpl>,
    mappings: HashMap<String, Box<Tokenizer + 'a>>,
}

impl<'a> Index<'a> {
    /// Sets the tokenizer to be used on content of the specified field.
    ///
    /// # Errors
    ///
    /// An [`error::IndexingError::MappingFieldAlreadyExists`] error is returned if a tokenizer is
    /// already set for the specified field.
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

    /// Adds the given document to the index.
    ///
    /// # Errors
    ///
    /// An [`error::IndexingError::MissingFieldMapping`] error is returned if the document contains
    /// a field that has no mapping defined.
    pub fn add_doc(&mut self, doc: &document::Document) -> IndexingResult<()> {
        for field in doc.fields() {
            if !self.mappings.contains_key(field.field) {
                return Err(error::IndexingError::MissingFieldMapping {
                    field: field.field.to_string(),
                });
            }
            let tokenizer = &self.mappings[field.field];
            for token in tokenizer.tokenize(field.value) {
                let posting = self.postings
                    .entry(format!("{}:{}", field.field, token.token))
                    .or_insert_with(posting_lists::new);
                posting.add_token(self.doc_id, token.position);
            }
        }
        self.doc_id += 1;
        Ok(())
    }

    /// Returns the posting lists associated with the given field.
    ///
    /// If the index does not have a posting lists for that field, then an [`posting_lists::empty`]
    /// posting is returned.
    pub fn get_postings_list(&self, field: &str) -> Box<&Posting> {
        if !self.postings.contains_key(field) {
            Box::new(posting_lists::empty())
        } else {
            Box::new(&self.postings[field])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokenizer::whitespace_tokenizer::WhiteSpaceTokenizer;

    #[test]
    fn should_return_empty_posting_on_unknown_field() {
        let index: Index = Default::default();

        let posting = index.get_postings_list("field1");
        assert_eq!(posting.len(), 0);

        let mut iter = posting.iter_docs();
        assert!(iter.next().is_none());

        let mut iter = posting.iter_docs_pos();
        assert!(iter.next().is_none());
    }

    #[test]
    fn should_fail_when_setting_two_times_a_mapping_field() {
        let mut index: Index = Default::default();

        let set_mapping_res = index.set_mapping(String::from("field1"), WhiteSpaceTokenizer::new());
        assert_eq!(set_mapping_res.is_ok(), true);

        let set_mapping_res = index.set_mapping(String::from("field1"), WhiteSpaceTokenizer::new());
        assert_eq!(set_mapping_res.is_err(), true);
    }

    #[test]
    fn should_fail_when_adding_doc_with_missing_field_mapping() {
        let mut index: Index = Default::default();

        let set_mapping_res = index.set_mapping(String::from("field1"), WhiteSpaceTokenizer::new());
        assert_eq!(set_mapping_res.is_ok(), true);

        let set_mapping_res = index.set_mapping(String::from("field1"), WhiteSpaceTokenizer::new());
        assert_eq!(set_mapping_res.is_err(), true);
    }

    /// Should index 2 docs over two postings list
    #[test]
    fn should_create_some_postings_list() {
        let mut index: Index = Default::default();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc: document::Document = Default::default();
        doc.add_field("field1", "aaa bbb aaa");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "bbb");
        index.add_doc(&doc).unwrap();

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
