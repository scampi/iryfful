use index::Index;
use index::posting_lists::DocItem;
use std::mem;
use std::u32::MAX;

pub mod query;

#[derive(Debug, PartialEq)]
pub struct SearchHit {
    doc_id: u32,
}

impl SearchHit {
    pub fn new(doc_id: u32) -> SearchHit {
        SearchHit { doc_id }
    }
}

impl DocItem for SearchHit {
    fn get_doc_id(&self) -> u32 {
        self.doc_id
    }
}

pub struct IndexSearcher<'a> {
    index: &'a Index<'a>,
}

impl<'q, 'a: 'q> IndexSearcher<'a> {
    pub fn new(index: &'a Index<'a>) -> IndexSearcher<'a> {
        IndexSearcher { index }
    }

    fn get_index(&self) -> &Index<'a> {
        self.index
    }

    pub fn search<T>(&'a self, query: &'q T) -> Box<Iterator<Item = SearchHit> + 'q>
    where
        T: query::Query,
    {
        Box::new(query.execute(self))
    }

    fn step_on_matching_doc<I, T>(&self, docs: Vec<Box<I>>) -> MatchingDocIterator<I, T>
    where
        I: Iterator<Item = T>,
        T: DocItem,
    {
        MatchingDocIterator { docs }
    }
}

struct MatchingDocIterator<I, T>
where
    I: Iterator<Item = T>,
    T: DocItem,
{
    docs: Vec<Box<I>>,
}

impl<I, T> Iterator for MatchingDocIterator<I, T>
where
    I: Iterator<Item = T>,
    T: DocItem,
{
    type Item = (u32, Vec<T>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut current_docs = Vec::with_capacity(self.docs.len());
        let mut max_doc_id = 0;
        let mut min_doc_id = MAX;

        // init the docs iteration
        for doc_iterator in self.docs.iter_mut() {
            match doc_iterator.next() {
                None => return None,
                Some(item) => {
                    if item.get_doc_id() > max_doc_id {
                        max_doc_id = item.get_doc_id();
                    } else if item.get_doc_id() < min_doc_id {
                        min_doc_id = item.get_doc_id();
                    }
                    current_docs.push(item);
                }
            }
        }

        if min_doc_id == max_doc_id {
            return Some((max_doc_id, current_docs));
        }

        // advance on the docs lists until a match is found
        'matching_loop: loop {
            for (i, doc_iterator) in self.docs.iter_mut().enumerate() {
                if current_docs[i].get_doc_id() == max_doc_id {
                    continue;
                }
                match doc_iterator.advance(max_doc_id) {
                    None => return None,
                    Some((found, doc)) => {
                        max_doc_id = doc.get_doc_id();
                        let _ = mem::replace(&mut current_docs[i], doc);
                        if !found {
                            continue 'matching_loop;
                        }
                    }
                }
            }
            // it's a match!
            return Some((max_doc_id, current_docs));
        }
    }
}

trait DocIterator: Iterator {
    fn advance(&mut self, doc_id: u32) -> Option<(bool, <Self as Iterator>::Item)>;
}

impl<I, T> DocIterator for I
where
    I: Iterator<Item = T>,
    T: DocItem,
{
    fn advance(&mut self, doc_id: u32) -> Option<(bool, <Self as Iterator>::Item)> {
        loop {
            match self.next() {
                None => return None,
                Some(item) => {
                    if item.get_doc_id() == doc_id {
                        return Some((true, item));
                    }
                    if item.get_doc_id() > doc_id {
                        return Some((false, item));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use index::document::Document;
    use index::posting_lists;
    use index::posting_lists::DocIdAndPosItem;
    use index::posting_lists::Posting;
    use tokenizer::whitespace_tokenizer::WhiteSpaceTokenizer;

    #[test]
    fn test_advance_doc() {
        let mut posting = posting_lists::new();
        posting.add_token(1, 42);
        posting.add_token(1, 45);
        posting.add_token(3, 1);
        posting.add_token(3, 2);
        posting.add_token(5, 3);
        posting.add_token(5, 33);
        posting.add_token(8, 6);
        posting.add_token(12, 4);

        let mut iter = posting.iter_docs();

        let next = iter.advance(3).unwrap();
        assert_eq!(next.0, true);
        assert_eq!(next.1.get_doc_id(), 3);

        let next = iter.advance(12).unwrap();
        assert_eq!(next.0, true);
        assert_eq!(next.1.get_doc_id(), 12);

        let next = iter.advance(15);
        assert_eq!(next.is_none(), true);
    }

    #[test]
    fn test_advance_doc_missing() {
        let mut posting = posting_lists::new();
        posting.add_token(1, 42);
        posting.add_token(1, 45);
        posting.add_token(3, 1);
        posting.add_token(3, 2);
        posting.add_token(5, 3);
        posting.add_token(5, 33);
        posting.add_token(8, 6);
        posting.add_token(12, 4);

        let mut iter = posting.iter_docs();

        let next = iter.advance(4).unwrap();
        assert_eq!(next.0, false);
        assert_eq!(next.1.get_doc_id(), 5);

        let next = iter.advance(15);
        assert_eq!(next.is_none(), true);
    }

    #[test]
    fn test_step_on_matching_doc_with_iter_docs() {
        // create index
        let mut index = Index::new();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "aaa bbb ccc");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "bbb");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "aaa ccc");
        index.add_doc(doc).unwrap();

        // get the postings lists for aaa and ccc
        let postings = ["aaa", "ccc"]
            .iter()
            .map(|term| {
                Box::new(
                    index
                        .get_postings_list(&format!("field1:{}", term))
                        .iter_docs(),
                )
            })
            .collect();
        let searcher = IndexSearcher::new(&index);
        let mut iter = searcher
            .step_on_matching_doc(postings)
            .map(|(doc_id, _)| doc_id);

        assert_eq!(iter.next().unwrap(), 0);
        assert_eq!(iter.next().unwrap(), 2);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_step_on_matching_doc_with_iter_docs_pos() {
        // create index
        let mut index = Index::new();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "aaa bbb ccc");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "bbb");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "aaa ccc");
        index.add_doc(doc).unwrap();

        // get the postings lists for aaa and ccc
        let postings = ["aaa", "ccc"]
            .iter()
            .map(|term| {
                Box::new(
                    index
                        .get_postings_list(&format!("field1:{}", term))
                        .iter_docs_pos(),
                )
            })
            .collect();
        let searcher = IndexSearcher::new(&index);
        let on_match = |(doc_id, docs): (u32, Vec<DocIdAndPosItem>)| {
            let mut diff = 0;
            for item in docs {
                if diff == 0 {
                    diff = item.positions[0];
                } else {
                    diff = item.positions[0] - diff;
                }
            }
            return if diff == 1 { Some(doc_id) } else { None };
        };
        let mut iter = searcher.step_on_matching_doc(postings).filter_map(on_match);

        assert_eq!(iter.next().unwrap(), 2);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_step_on_matching_doc() {
        // create index
        let mut index = Index::new();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "aaa ccc");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "aaa bbb");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "bbb ccc");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "aaa bbb");
        index.add_doc(doc).unwrap();

        // get the postings lists for aaa and bbb
        let postings = ["aaa", "bbb"]
            .iter()
            .map(|term| {
                Box::new(
                    index
                        .get_postings_list(&format!("field1:{}", term))
                        .iter_docs(),
                )
            })
            .collect();
        let searcher = IndexSearcher::new(&index);
        let mut iter = searcher
            .step_on_matching_doc(postings)
            .map(|(doc_id, _)| doc_id);

        assert_eq!(iter.next().unwrap(), 1);
        assert_eq!(iter.next().unwrap(), 3);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_step_on_matching_doc_advance() {
        // create index
        let mut index = Index::new();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "aaa");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "bbb");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "aaa bbb");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "aaa bbb");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "aaa");
        index.add_doc(doc).unwrap();

        let mut doc = Document::new();
        doc.add_field("field1", "bbb");
        index.add_doc(doc).unwrap();

        // get the postings lists for aaa and bbb
        let postings = ["aaa", "bbb"]
            .iter()
            .map(|term| {
                Box::new(
                    index
                        .get_postings_list(&format!("field1:{}", term))
                        .iter_docs(),
                )
            })
            .collect();
        let searcher = IndexSearcher::new(&index);
        let mut iter = searcher
            .step_on_matching_doc(postings)
            .map(|(doc_id, _)| doc_id);

        assert_eq!(iter.next().unwrap(), 2);
        assert_eq!(iter.next().unwrap(), 3);
        assert_eq!(iter.next(), None);
    }
}
