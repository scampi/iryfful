//! Define what to search for in a document.
//!
//! The following queries can be executed over and index via an [`IndexSearcher`]:
//! - a [`boolean query`][boolean]: a boolean combination of other queries.
//! - a [`phrase query`][phrase]: match documents that have a specific sequence of terms.
//! - a [`term query`][term]: match documents that have a specific term occurring.
//!
//! [boolean]: boolean_query/index.html
//! [phrase]: phrase_query/index.html
//! [term]: term_query/index.html
use super::IndexSearcher;
use super::SearchHit;
use std::fmt::Debug;

pub mod boolean_query;
pub mod phrase_query;
pub mod term_query;

/// The `Query` type filters an index and returns an [`Iterator`] of matching documents.
pub trait Query: Debug {
    /// Retain matching document from the given index.
    fn execute<'q, 'i: 'q>(
        &'q self,
        index_search: &'i IndexSearcher,
    ) -> Box<Iterator<Item = SearchHit> + 'q>;
}
