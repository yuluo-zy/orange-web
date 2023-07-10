use std::{cell::UnsafeCell, sync::atomic::Ordering};
use crate::router::tree::art::node::{Node, NodeTrait};
use crate::router::tree::art::node::keys::Partial;
use crate::router::tree::art::utils::TreeError;

pub(crate) struct ReadGuard<'a,P: Partial,V> {
    version: usize,
    node: &'a UnsafeCell<Node<P,V>>,
}

impl<'a,P: Partial,V> ReadGuard<'a, P, V> {
    pub(crate) fn new(v: usize, node: &'a Node<P,V>) -> Self {
        Self {
            version: v,
            node: unsafe { &*(node as *const Node<P,V> as *const UnsafeCell<Node<P,V>>) },
        }
    }

    pub(crate) fn as_ref(&self) -> &'a Node<P,V> {
        unsafe { &*self.node.get() }
    }

    pub(crate) fn as_mut(&self) -> &mut Node<P,V> {
        unsafe { self.node.get() as &mut Node<P,V> }
    }

    pub(crate) fn check_version(&self) -> Result<usize, TreeError> {
        let v = self
            .as_ref()
            .type_version_lock_obsolete
            .load(Ordering::Acquire);

        if v == self.version {
            Ok(v)
        } else {
            Err(TreeError::VersionNotMatch)
        }
    }

    pub(crate) fn unlock(self) -> Result<usize, TreeError> {
        self.check_version()
    }

    pub(crate) fn upgrade(self) -> Result<WriteGuard<'a, P,V>, (Self, TreeError)> {
        let new_version = self.version + 0b10;
        match self
            .as_ref()
            .type_version_lock_obsolete
            .compare_exchange_weak(
                self.version,
                new_version,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
            Ok(_) => Ok(WriteGuard {
                node: unsafe { &mut *self.node.get() },
            }),
            Err(v) => Err((self, TreeError::VersionNotMatch)),
        }
    }
}


pub(crate) struct WriteGuard<'a, P: Partial,V> {
    node: &'a mut Node<P, V>,
}

impl<'a,P: Partial,V> WriteGuard<'a, P,V> {
    pub(crate) fn as_ref(&self) -> &Node<P, V> {
        self.node
    }

    pub(crate) fn as_mut(&mut self) -> &mut Node<P, V> {
        self.node
    }

    pub(crate) fn mark_obsolete(&mut self) {
        self.node
            .type_version_lock_obsolete
            .fetch_add(0b01, Ordering::Release);
    }
}

impl<'a,P: Partial, V> Drop for WriteGuard<'a, P,V> {
    fn drop(&mut self) {
        self.node
            .type_version_lock_obsolete
            .fetch_add(0b10, Ordering::Release);
    }
}
