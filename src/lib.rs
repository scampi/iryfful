extern crate failure;
#[macro_use]
extern crate failure_derive;

#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;

pub mod index;
pub mod search;
pub mod tokenizer;
