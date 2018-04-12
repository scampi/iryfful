//! Logic for creating a posting lists and interacting with it.
use std::fmt::Debug;
use std::iter;

/// The `Posting` type allows to add tokens to the index and then iterating over them
pub trait Posting {
    /// Returns `true` if there is no token in this posting lists.
    fn is_empty(&self) -> bool;

    /// Returns the number of documents this posting lists contains.
    fn len(&self) -> usize;

    /// Adds a token to this posting list with the given document ID occurring at position within
    /// that document.
    fn add_token(&mut self, doc_id: u32, position: u32);

    /// Creates an iterator over [`DocIdItem`]s.
    fn iter_docs<'a>(&'a self) -> Box<Iterator<Item = DocIdItem> + 'a>;

    /// Creates an iterator over [`DocIdAndPosItem`]s.
    fn iter_docs_pos<'a>(&'a self) -> Box<Iterator<Item = DocIdAndPosItem<'a>> + 'a>;
}

/// Creates a new [`Posting`] instance.
pub fn new() -> PostingImpl {
    PostingImpl {
        docs: Vec::new(),
        positions: Vec::new(),
    }
}

static EMPTY: EmptyPosting = EmptyPosting {};

/// Returns a [`Posting`] instance which is always empty.
pub fn empty() -> &'static EmptyPosting {
    &EMPTY
}

pub struct EmptyPosting;

impl Posting for EmptyPosting {
    fn is_empty(&self) -> bool {
        true
    }

    fn len(&self) -> usize {
        0
    }

    fn add_token(&mut self, _doc_id: u32, _position: u32) {}

    fn iter_docs<'a>(&'a self) -> Box<Iterator<Item = DocIdItem> + 'a> {
        Box::new(iter::empty::<DocIdItem>())
    }

    fn iter_docs_pos<'a>(&'a self) -> Box<Iterator<Item = DocIdAndPosItem<'a>> + 'a> {
        Box::new(iter::empty::<DocIdAndPosItem>())
    }
}

#[derive(Debug)]
pub struct PostingImpl {
    docs: Vec<DocEntry>,
    positions: Vec<u32>,
}

#[derive(Debug)]
struct DocEntry {
    doc_id: u32,
    freqs: u32,
    positions_offset: u32,
}

impl DocEntry {
    fn new(doc_id: u32, positions_offset: u32) -> DocEntry {
        DocEntry {
            doc_id,
            freqs: 0,
            positions_offset,
        }
    }
}

/// Interface for any [`Iterator`]'s item which iterates over a list of documents.
pub trait DocItem: Debug {
    /// Returns the document ID of this item.
    fn get_doc_id(&self) -> u32;
}

/// A [`DocItem`] which provides only the ID of a document.
#[derive(Debug)]
pub struct DocIdItem {
    doc_id: u32,
}

impl DocItem for DocIdItem {
    fn get_doc_id(&self) -> u32 {
        self.doc_id
    }
}

/// A [`DocItem`] which provides in addition to a document's ID the positions of each token's occurrence.
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

impl Posting for PostingImpl {
    fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }

    fn len(&self) -> usize {
        self.docs.len()
    }

    fn add_token(&mut self, doc_id: u32, position: u32) {
        let create_doc_posting = match self.docs.last() {
            None => true,
            Some(doc_posting) if doc_posting.doc_id != doc_id => true,
            Some(_) => false,
        };

        if create_doc_posting {
            self.docs
                .push(DocEntry::new(doc_id, self.positions.len() as u32));
        }

        let doc_posting = self.docs
            .last_mut()
            .expect("could not get the last doc posting");
        doc_posting.freqs += 1;
        self.positions.push(position);
    }

    fn iter_docs<'a>(&'a self) -> Box<Iterator<Item = DocIdItem> + 'a> {
        Box::new(self.docs.iter().map(|doc| DocIdItem { doc_id: doc.doc_id }))
    }

    fn iter_docs_pos<'a>(&'a self) -> Box<Iterator<Item = DocIdAndPosItem<'a>> + 'a> {
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
        let mut posting = new();
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
        let mut posting = new();
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
        let mut posting = new();
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
