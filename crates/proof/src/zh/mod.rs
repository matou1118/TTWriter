//! Chinese proofing. Chinese doesn't spell-check word-by-word (there is no dictionary of
//! "correct" characters the way alphabetic languages have a dictionary of correct words), so
//! there is no spelling module. Grammar rules catch repeated characters and spacing issues
//! around CJK punctuation. Hyphenation keeps the existing fallback.

pub mod grammar;
