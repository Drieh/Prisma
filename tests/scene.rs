use prismae::node::NodeID;
use prismae::{Scene, error::PrismaError};

#[test]
fn scene_contains() {
    let mut scene = Scene::new();
    let id = scene.new_node().get_id();

    assert!(scene.contains(id));
    assert!(!scene.contains(NodeID::id(123)));
}

#[test]
fn get_node() {
    let mut scene = Scene::new();
    let id = scene.new_node().get_id();

    assert!(scene.get_node(id).is_ok());

    assert_eq!(
        scene.get_node(NodeID::id(123)).unwrap_err(),
        PrismaError::NodeNotFound(NodeID::id(123))
    );
}

#[test]
fn store_and_recover_state() {
    let mut scene = Scene::new();

    let mut node = scene.new_node();

    node.set_state("key", 1);

    assert!(node.get_state::<i32>("key").is_ok());
    assert_eq!(*node.get_state::<i32>("key").unwrap(), 1);
    assert!(node.get_state::<i32>("key2").is_err());
}
