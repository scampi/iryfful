use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Document<'a> {
    fields: HashMap<&'a str, Vec<&'a str>>,
}

impl<'a> Document<'a> {
    pub fn clear(&mut self) {
        self.fields.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn add_field(&mut self, field: &'a str, value: &'a str) {
        self.fields
            .entry(field)
            .or_insert_with(Vec::new)
            .push(value);
    }

    pub fn fields(&'a self) -> Box<Iterator<Item = Content<'a>> + 'a> {
        Box::new(
            self.fields.iter().flat_map(|(field, values)| {
                values.iter().map(move |value| Content::new(field, value))
            }),
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Content<'a> {
    pub field: &'a str,
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
