/// Internal string interning store.
mod store;
/// The `ShareableString` type.
mod string;
/// A map for storing translations of `ShareableString`s.
mod translation_map;

pub use store::*;
pub use string::*;
pub use translation_map::*;
