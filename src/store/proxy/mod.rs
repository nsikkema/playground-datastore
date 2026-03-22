/// Proxy for accessing basic data.
mod basic_proxy;
/// Proxy for accessing container data.
mod container_proxy;
/// Proxy for accessing object data.
mod object_proxy;
/// Proxy for accessing table data.
mod table_proxy;

pub use basic_proxy::BasicProxy;
pub use container_proxy::ContainerProxy;
pub use object_proxy::ObjectProxy;
pub use table_proxy::TableProxy;
