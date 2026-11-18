pub mod manager;

mod macros;
mod registry;
mod resource;

pub use resource::*;

pub(crate) use registry::ResourceEntry;
