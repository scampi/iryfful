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

    pub fn iter_docs<'a>(&'a self) -> Box<Iterator<Item = u32> + 'a> {
        Box::new(self.docs.iter().map(|doc| doc.doc_id))
    }
}

#[cfg(test)]
mod tests {
    use expectest::prelude::*;
    use super::*;

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

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(1));

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(3));

        let next_doc = iter.next();
        expect!(next_doc).to(be_none());
    }
}
