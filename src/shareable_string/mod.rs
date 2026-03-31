/// Internal string interning store.
pub mod store;
/// The `ShareableString` type.
pub mod string;
/// A map for storing translations of `ShareableString`s.
pub mod translation_map;

pub use store::*;
pub use string::*;
pub use translation_map::*;
