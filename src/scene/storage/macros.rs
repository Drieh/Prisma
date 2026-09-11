macro_rules! define_handlers {
    (
        $(
            {
                $handler:ident, $storage:ident
            }
        )*
    ) => {
        $(
            pub struct $handler<'a> {
                pub(crate) storage: &'a mut HashMap<NodeID, $storage>,
            }
            impl<'a> $handler<'a> {
                pub(crate) fn new(storage: &'a mut HashMap<NodeID, $storage>) -> Self {
                    Self { storage }
                }

                pub fn get(&self, id: NodeID) -> Result<&$storage, PrismaError> {
                    self.storage
                        .get(&id)
                        .ok_or(PrismaError::NodeComponentNotFound(id))
                }

                pub fn contains(&self, id: NodeID) -> bool {
                    self.storage.contains_key(&id)
                }

                pub(crate) fn get_unchecked(&self, id: NodeID) -> &$storage {
                    self.storage.get(&id).expect("Node component not found!")
                }

                pub(crate) fn get_unchecked_mut(&mut self, id: NodeID) -> &mut $storage {
                    self.storage
                        .get_mut(&id)
                        .expect("Node component not found!")
                }

                pub(crate) fn insert(&mut self, id: NodeID) {
                    if self.contains(id) {
                        panic!("Node component already exists!");
                    }
                    self.storage.insert(id, $storage::new());
                }

                pub(crate) fn set(&mut self, id: NodeID, component: $storage) {
                    self.storage.insert(id, component);
                }

                pub(crate) fn remove(&mut self, id: NodeID) -> $storage {
                    self.storage.remove(&id).expect("Node component not found!")
                }
            }
        )*

    };
}
pub(crate) use define_handlers;
