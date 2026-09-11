use std::collections::{HashMap, VecDeque};

use crate::node::{Action, ActionOrigin, ActionType, node_action::ActionFamily};

pub struct NodeQueue {
    queue: HashMap<ActionType, VecDeque<Action>>,
}

impl NodeQueue {
    pub fn new() -> Self {
        Self {
            queue: HashMap::new(),
        }
    }

    pub(crate) fn take<T>(&mut self) -> VecDeque<T>
    where
        T: ActionFamily,
    {
        let mut list: VecDeque<T> = VecDeque::new();
        loop {
            let action = self.pop_front::<T>();
            if action.is_some() {
                list.push_back(action.unwrap());
            }
            break list;
        }
    }

    pub(crate) fn push_front<T>(&mut self, action: T, origin: ActionOrigin)
    where
        T: ActionFamily,
    {
        self.queue
            .entry(T::TYPE)
            .or_default()
            .push_front(T::into_action(action, origin));
    }

    pub(crate) fn push_back<T>(&mut self, action: T, origin: ActionOrigin)
    where
        T: ActionFamily,
    {
        self.queue
            .entry(T::TYPE)
            .or_default()
            .push_back(T::into_action(action, origin));
    }

    pub(crate) fn pop_front<T>(&mut self) -> Option<T>
    where
        T: ActionFamily,
    {
        self.queue
            .get_mut(&T::TYPE)?
            .pop_front()
            .and_then(|action| T::cast(action))
    }

    pub(crate) fn pop_front_action<T>(&mut self) -> Option<Action>
    where
        T: ActionFamily,
    {
        self.queue.get_mut(&T::TYPE)?.pop_front()
    }
}
