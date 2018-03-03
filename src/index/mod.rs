use std::collections::HashMap;
use tokenizer::Tokenizer;

pub mod posting_lists;

#[derive(Debug)]
pub struct Index {
    doc_id: u32,
    postings: HashMap<String, posting_lists::Posting>,
}

impl Index {
    pub fn new() -> Index {
        Index {
            doc_id: 0,
            postings: HashMap::new(),
        }
    }

    pub fn add_doc<'a, T>(&mut self, mut tokenizer: Tokenizer<'a, T>)
    where
        T: Iterator<Item = &'a str>,
    {
        loop {
            let token = match tokenizer.next() {
                None => break,
                Some(token) => token,
            };
            let posting = self.postings
                .entry(token.token)
                .or_insert(posting_lists::Posting::new());
            posting.add_token(self.doc_id, token.position);
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
        let data = "aaa bbb aaa";
        let white_space_tokenizer = WhiteSpaceTokenizer::new(data);

        let mut index = Index::new();
        index.add_doc(white_space_tokenizer);

        let data = "bbb";
        let white_space_tokenizer = WhiteSpaceTokenizer::new(data);
        index.add_doc(white_space_tokenizer);

        assert_eq!(index.postings.len(), 2);
        for (key, posting) in index.postings.iter() {
            match key.as_ref() {
                "aaa" => assert_eq!(posting.len(), 1),
                "bbb" => assert_eq!(posting.len(), 2),
                _ => panic!(format!("got unexpected key={}", key)),
            }
        }
    }
}
