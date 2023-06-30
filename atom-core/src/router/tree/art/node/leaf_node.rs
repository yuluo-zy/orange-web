use crate::router::tree::art::node::partials::Partial;

pub struct NodeLeaf<K: Partial, V> {
    pub kay: K,
    pub value: Option<V>,
}

impl<K: Partial, V> NodeLeaf<K, V> {
    #[inline]
    pub fn value_ref(&self) -> Option<&V> {
        self.value.as_ref()
    }
    #[inline]
    pub fn value_mut(&mut self) -> Option<&mut V> {
        self.value.as_mut()
    }
}