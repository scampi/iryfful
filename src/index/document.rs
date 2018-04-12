//! Document representation for the purpose of indexing.
use std::collections::HashMap;

/// A document is designed as a multi-valued list of fields.
#[derive(Debug, Default)]
pub struct Document<'a> {
    fields: HashMap<&'a str, Vec<&'a str>>,
}

impl<'a> Document<'a> {
    /// Removes all values within this document.
    pub fn clear(&mut self) {
        self.fields.clear();
    }

    /// Returns `true` if this document has no content.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Returns the number of fields this document contains.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Adds a field-value pair to this document.
    pub fn add_field(&mut self, field: &'a str, value: &'a str) {
        self.fields
            .entry(field)
            .or_insert_with(Vec::new)
            .push(value);
    }

    /// Returns an [`Iterator`] over this document's content.
    ///
    /// A [`Content`] represents a field-value pair.
    pub fn fields(&'a self) -> Box<Iterator<Item = Content<'a>> + 'a> {
        Box::new(
            self.fields.iter().flat_map(|(field, values)| {
                values.iter().map(move |value| Content::new(field, value))
            }),
        )
    }
}

/// A type containing a pair of a field with one of its values.
#[derive(Debug, PartialEq)]
pub struct Content<'a> {
    /// The field name.
    pub field: &'a str,
    /// The content associated to that field.
    pub value: &'a str,
}

impl<'a> Content<'a> {
    fn new(field: &'a str, value: &'a str) -> Content<'a> {
        Content { field, value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_multi_valued_document() {
        let mut doc: Document = Default::default();
        doc.add_field("field1", "aaa");
        doc.add_field("field1", "bbb");
        doc.add_field("field2", "ccc");

        assert_eq!(doc.len(), 2);
        let mut n_values = 0;
        for field in doc.fields() {
            if field == Content::new("field1", "aaa") || field == Content::new("field1", "bbb")
                || field == Content::new("field2", "ccc")
            {
                n_values += 1;
            }
        }
        assert_eq!(n_values, 3);
    }
}
