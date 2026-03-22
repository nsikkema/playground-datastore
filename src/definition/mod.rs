/// Definitions for basic data types (strings, numbers, etc.).
mod basic_definition;
/// Definitions for map-based data structures.
mod map_definition;
/// Definitions for object-based data structures.
mod object_definition;
/// Definitions for properties within objects or containers.
mod property_definition;
/// Definitions for struct-like data structures.
mod struct_definition;
/// Definitions for table-based data structures.
mod table_definition;

pub use basic_definition::*;
pub use map_definition::*;
pub use object_definition::*;
pub use property_definition::*;
pub use struct_definition::*;
pub use table_definition::*;
