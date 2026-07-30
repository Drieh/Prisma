pub mod node;
#[warn(clippy::module_inception)]
mod scene;
pub(crate) mod storage;

pub use node::NodeID;
pub use node::NodeView;
pub use scene::Scene;
