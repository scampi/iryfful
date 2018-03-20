use index::Index;
use index::posting_lists::DocItem;

pub mod query;

pub struct IndexSearcher<'a> {
    index: &'a Index<'a>,
}

impl<'a> IndexSearcher<'a> {
    pub fn new(index: &'a Index<'a>) -> IndexSearcher<'a> {
        IndexSearcher { index }
    }

    fn get_index(&self) -> &Index<'a> {
        self.index
    }

    pub fn search<'q, T>(&'a self, query: &'q T) -> Box<Iterator<Item = u32> + 'a>
    where
        T: query::Query<'q, 'a> + 'q,
    {
        Box::new(query.execute(&self))
    }

    pub fn step_on_matching_doc<I, T, F>(&self, mut postings: Vec<Box<I>>, mut f: F)
    where
        I: Iterator<Item = T>,
        T: DocItem,
        F: FnMut(u32, &[T]),
    {
        let mut current_docs = Vec::with_capacity(postings.len());

        loop {
            // get the current docs
            current_docs.clear();
            for posting in postings.iter_mut() {
                match posting.next() {
                    None => return,
                    Some(doc) => current_docs.push(doc),
                }
            }
            // check if the docs have the same ids
            let max_doc_id = current_docs.iter().map(|doc| doc.get_doc_id()).max();
            let min_doc_id = current_docs.iter().map(|doc| doc.get_doc_id()).min();
            if max_doc_id == min_doc_id {
                // apply the operation on success
                f(max_doc_id.unwrap(), &current_docs[..]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokenizer::whitespace_tokenizer::WhiteSpaceTokenizer;
    use super::*;
    use index::document::Document;
    use index::posting_lists::DocIdItem;
    use index::posting_lists::DocIdAndPosItem;

    #[test]
    fn test_step_on_matching_doc_with_iter_docs() {
        // create index
        let mut index = Index::new();
        index.set_mapping(String::from("field1"), WhiteSpaceTokenizer::new());

        let mut doc = Document::new();
        doc.add_field("field1", "aaa bbb ccc");
        index.add_doc(doc);

        let mut doc = Document::new();
        doc.add_field("field1", "bbb");
        index.add_doc(doc);

        let mut doc = Document::new();
        doc.add_field("field1", "aaa ccc");
        index.add_doc(doc);

        // get the postings lists for aaa and ccc
        let postings = ["aaa", "ccc"]
            .iter()
            .map(|term| {
                Box::new(
                    index
                        .get_postings_list(&format!("field1:{}", term))
                        .unwrap()
                        .iter_docs(),
                )
            })
            .collect();
        let searcher = IndexSearcher::new(&index);
        let mut matches = Vec::new();
        {
            let on_match = |doc_id: u32, _: &[DocIdItem]| matches.push(doc_id);
            searcher.step_on_matching_doc(postings, on_match);
        }

        assert_eq!(matches.len(), 2);
        assert_eq!(matches.get(0).unwrap(), &0);
        assert_eq!(matches.get(1).unwrap(), &2);
    }

    #[test]
    fn test_step_on_matching_doc_with_iter_docs_pos() {
        // create index
        let mut index = Index::new();
        index.set_mapping(String::from("field1"), WhiteSpaceTokenizer::new());

        let mut doc = Document::new();
        doc.add_field("field1", "aaa bbb ccc");
        index.add_doc(doc);

        let mut doc = Document::new();
        doc.add_field("field1", "bbb");
        index.add_doc(doc);

        let mut doc = Document::new();
        doc.add_field("field1", "aaa ccc");
        index.add_doc(doc);

        // get the postings lists for aaa and ccc
        let postings = ["aaa", "ccc"]
            .iter()
            .map(|term| {
                Box::new(
                    index
                        .get_postings_list(&format!("field1:{}", term))
                        .unwrap()
                        .iter_docs_pos(),
                )
            })
            .collect();
        let searcher = IndexSearcher::new(&index);
        let mut matches = Vec::new();
        {
            let on_match = |doc_id: u32, docs: &[DocIdAndPosItem]| {
                let mut diff = 0;
                for item in docs.iter() {
                    if diff == 0 {
                        diff = item.positions[0];
                    } else {
                        diff = item.positions[0] - diff;
                    }
                }
                if diff == 1 {
                    matches.push(doc_id);
                }
            };
            searcher.step_on_matching_doc(postings, on_match);
        }

        assert_eq!(matches.len(), 1);
        assert_eq!(matches.get(0).unwrap(), &2);
    }
}
