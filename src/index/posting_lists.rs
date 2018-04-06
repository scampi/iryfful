use std::fmt::Debug;

#[derive(Debug)]
pub struct Posting {
    docs: Vec<DocPosting>,
    positions: Vec<u32>,
}

#[derive(Debug)]
struct DocPosting {
    doc_id: u32,
    freqs: u32,
    positions_offset: u32,
}

impl DocPosting {
    fn new(doc_id: u32, positions_offset: u32) -> DocPosting {
        DocPosting {
            doc_id,
            freqs: 0,
            positions_offset,
        }
    }
}

pub trait DocItem: Debug {
    fn get_doc_id(&self) -> u32;
}

#[derive(Debug)]
pub struct DocIdItem {
    doc_id: u32,
}

impl DocItem for DocIdItem {
    fn get_doc_id(&self) -> u32 {
        self.doc_id
    }
}

#[derive(Debug)]
pub struct DocIdAndPosItem<'a> {
    doc_id: u32,
    pub positions: &'a [u32],
}

impl<'a> DocItem for DocIdAndPosItem<'a> {
    fn get_doc_id(&self) -> u32 {
        self.doc_id
    }
}

impl Posting {
    pub fn new() -> Posting {
        Posting {
            docs: Vec::new(),
            positions: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.docs.len()
    }

    pub fn add_token(&mut self, doc_id: u32, position: u32) {
        // no doc in this posting
        if self.docs.is_empty() {
            self.docs
                .push(DocPosting::new(doc_id, self.positions.len() as u32));
        }

        // check if the last doc of the posting is the same as the one passed in argument
        let mut is_new_doc = false;
        if let Some(doc_posting) = self.docs.last() {
            if doc_posting.doc_id != doc_id {
                is_new_doc = true;
            }
        }
        if is_new_doc {
            self.docs
                .push(DocPosting::new(doc_id, self.positions.len() as u32));
        }

        // update the positing list with the passed token
        let doc_posting = self.docs
            .last_mut()
            .expect("could not get the last doc posting");
        doc_posting.freqs += 1;
        self.positions.push(position);
    }

    pub fn iter_docs<'a>(&'a self) -> Box<Iterator<Item = DocIdItem> + 'a> {
        Box::new(self.docs.iter().map(|doc| DocIdItem { doc_id: doc.doc_id }))
    }

    pub fn iter_docs_pos<'a>(&'a self) -> Box<Iterator<Item = DocIdAndPosItem<'a>> + 'a> {
        Box::new(self.docs.iter().map(move |doc| {
            let start = doc.positions_offset as usize;
            let end = (doc.positions_offset + doc.freqs) as usize;
            DocIdAndPosItem {
                doc_id: doc.doc_id,
                positions: &self.positions[start..end],
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;

    #[test]
    fn should_add_tokens() {
        let mut posting = Posting::new();
        posting.add_token(1, 42);
        posting.add_token(1, 45);
        posting.add_token(3, 2);

        expect!(posting.docs.iter()).to(have_count(2));
        expect!(posting.positions.iter()).to(have_count(3));

        for doc in posting.docs.iter() {
            match doc.doc_id {
                1 => {
                    assert_eq!(doc.freqs, 2);
                    assert_eq!(doc.positions_offset, 0);
                    assert_eq!(posting.positions[0], 42);
                    assert_eq!(posting.positions[1], 45);
                }
                3 => {
                    assert_eq!(doc.freqs, 1);
                    assert_eq!(doc.positions_offset, 2);
                    assert_eq!(posting.positions[2], 2);
                }
                _ => panic!(format!("got unexpected doc with id={}", doc.doc_id)),
            }
        }
    }

    #[test]
    fn test_iter_docs() {
        let mut posting = Posting::new();
        posting.add_token(1, 42);
        posting.add_token(1, 45);
        posting.add_token(3, 2);

        let mut iter = posting.iter_docs();

        let next = iter.next();
        assert_eq!(next.unwrap().doc_id, 1);

        let next = iter.next();
        assert_eq!(next.unwrap().doc_id, 3);

        let next = iter.next();
        assert_eq!(next.is_none(), true);
    }

    #[test]
    fn test_iter_docs_pos() {
        let mut posting = Posting::new();
        posting.add_token(1, 42);
        posting.add_token(1, 45);
        posting.add_token(3, 2);

        let mut iter = posting.iter_docs_pos();

        let next = iter.next().unwrap();
        assert_eq!(next.doc_id, 1);
        assert_eq!(next.positions.len(), 2);
        assert_eq!(next.positions.get(0), Some(&42));
        assert_eq!(next.positions.get(1), Some(&45));
        assert_eq!(next.positions.get(2).is_none(), true);

        let next = iter.next().unwrap();
        assert_eq!(next.doc_id, 3);
        assert_eq!(next.positions.len(), 1);
        assert_eq!(next.positions.get(0), Some(&2));
        assert_eq!(next.positions.get(1).is_none(), true);

        let next = iter.next();
        assert_eq!(next.is_none(), true);
    }
}
