//! Match a document that have terms juxtaposing within a configurable slop in any order.
//!
//! Set the slop to a value greater or equal to 1 in order to configure the maximum distance
//! between two terms.
//!
//! # Examples
//!
//! ```no_run
//! use ::iryfful::search::query::phrase_query::PhraseQuery;
//!
//! // match the phrase "aaa bbb" occurring within the field "field1", with at most a distance of 2
//! // inbetween.
//! // this would match "something aaa bbb other" and "something aaa ccc bbb other" but not "aaa
//! // ccc ddd bbb"
//! let mut pq = PhraseQuery::new("field1", vec!["aaa", "bbb"]);
//! pq.set_slop(2);
//! ```
use super::Query;
use index::posting_lists::DocIdAndPosItem;
use search::IndexSearcher;
use search::SearchHit;

#[derive(Debug)]
pub struct PhraseQuery<'a> {
    field: &'a str,
    terms: Vec<&'a str>,
    slop: u8,
}

impl<'a> PhraseQuery<'a> {
    /// Creates a new phrase query for specified sequence of terms. The order of terms is not
    /// relevant while matching, but is while scoring.
    pub fn new(field: &'a str, terms: Vec<&'a str>) -> PhraseQuery<'a> {
        PhraseQuery {
            field,
            terms,
            slop: 1,
        }
    }

    /// Defines the maximum distance separating two terms.
    pub fn set_slop(&mut self, slop: u8) {
        self.slop = slop;
    }
}

impl<'pq> Query for PhraseQuery<'pq> {
    fn execute<'q, 'i: 'q>(
        &'q self,
        index_search: &'i IndexSearcher,
    ) -> Box<Iterator<Item = SearchHit> + 'q> {
        let postings = self.terms
            .iter()
            .map(|term| {
                Box::new(
                    index_search
                        .get_index()
                        .get_postings_list(&format!("{}:{}", self.field, term))
                        .iter_docs_pos(),
                )
            })
            .collect();
        let mut positions = Vec::with_capacity(self.terms.len());
        let on_match = move |(doc_id, terms): (u32, Vec<DocIdAndPosItem>)| {
            let term1 = &terms[0];
            let terms_rest = &terms[1..];
            let fit = |positions: &Vec<u32>, posx: &u32| {
                for pos in positions.iter() {
                    if pos != posx && (*pos as i32 - *posx as i32).abs() as u8 <= self.slop {
                        return true;
                    }
                }
                false
            };
            let past_all_positions = |positions: &Vec<u32>, posx: &u32| {
                for pos in positions.iter() {
                    if posx <= pos {
                        return false;
                    }
                }
                true
            };

            // Algorithm mostly taken from https://nlp.stanford.edu/IR-book/html/htmledition/positional-indexes-1.html
            for pos1 in term1.positions.iter() {
                positions.clear();
                positions.push(*pos1);

                // in case there is only one term, there is no need to have another go at the
                // positions to see if any valid combination still exists
                let mut checked_all = terms_rest.len() == 1;
                // because the match of terms can be done in any order, we may need to iterate
                // the terms several times
                loop {
                    let candidates_count = positions.len();
                    for termx in terms_rest.iter() {
                        for posx in termx.positions.iter() {
                            if fit(&positions, posx) {
                                positions.push(*posx);
                                // TODO: should not break here so that all occurring phrases
                                // are found.
                                // matching phrases should be added to a list that could be
                                // used for scoring.
                                break;
                            } else if past_all_positions(&positions, posx) {
                                break;
                            }
                        }
                        if positions.len() == terms.len() {
                            // match
                            // TODO: a single match of the term is enough until the fix to
                            // gather all occurring phrases is done
                            return Some(SearchHit::new(doc_id));
                        }
                    }
                    if checked_all && candidates_count == positions.len() {
                        // no more matches in any order
                        break;
                    }
                    checked_all = true;
                }
            }
            None
        };

        Box::new(index_search.conjunction(postings).filter_map(on_match))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;
    use index::Index;
    use index::document::Document;
    use search::IndexSearcher;
    use search::SearchHit;
    use tokenizer::whitespace_tokenizer::WhiteSpaceTokenizer;

    #[test]
    fn test_two_terms() {
        let mut index: Index = Default::default();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        // match at the start
        let mut doc: Document = Default::default();
        doc.add_field("field1", "aaa bbb ccc aaa");
        index.add_doc(&doc).unwrap();

        // match at the end
        doc.clear();
        doc.add_field("field1", "aaa ccc aaa bbb");
        index.add_doc(&doc).unwrap();

        // order of the terms is not important
        doc.clear();
        doc.add_field("field1", "aaa ccc bbb aaa");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa ccc bbb");
        index.add_doc(&doc).unwrap();

        let index_search = &IndexSearcher::new(&index);

        let pq = PhraseQuery::new("field1", vec!["aaa", "bbb"]);
        let mut iter = pq.execute(index_search);

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(SearchHit::new(0)));

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(SearchHit::new(1)));

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(SearchHit::new(2)));

        let next_doc = iter.next();
        expect!(next_doc).to(be_none());
    }

    #[test]
    fn test_three_terms() {
        let mut index: Index = Default::default();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc: Document = Default::default();
        doc.add_field("field1", "aaa bbb ccc");
        index.add_doc(&doc).unwrap();

        // order of the terms is not important
        doc.clear();
        doc.add_field("field1", "bbb aaa ccc");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa aaa bbb ccc");
        index.add_doc(&doc).unwrap();

        let index_search = &IndexSearcher::new(&index);

        let pq = PhraseQuery::new("field1", vec!["aaa", "bbb", "ccc"]);
        let mut iter = pq.execute(index_search);

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(SearchHit::new(0)));

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(SearchHit::new(1)));

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(SearchHit::new(2)));

        let next_doc = iter.next();
        expect!(next_doc).to(be_none());
    }

    #[test]
    fn test_slop() {
        let mut index: Index = Default::default();
        index
            .set_mapping(String::from("field1"), WhiteSpaceTokenizer::new())
            .unwrap();

        let mut doc: Document = Default::default();
        doc.add_field("field1", "aaa ccc bbb");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "bbb ccc aaa");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "bbb ccc ddd aaa");
        index.add_doc(&doc).unwrap();

        doc.clear();
        doc.add_field("field1", "aaa bbb");
        index.add_doc(&doc).unwrap();

        let index_search = &IndexSearcher::new(&index);

        let mut pq = PhraseQuery::new("field1", vec!["aaa", "bbb"]);
        pq.set_slop(2);

        let mut iter = pq.execute(index_search);

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(SearchHit::new(0)));

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(SearchHit::new(1)));

        let next_doc = iter.next();
        expect!(next_doc).to(be_some().value(SearchHit::new(3)));

        let next_doc = iter.next();
        expect!(next_doc).to(be_none());
    }
}
