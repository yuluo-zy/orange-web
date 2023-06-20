use std::marker::PhantomData;
use std::sync::Arc;
use crate::router::tree::art::node::node::Node256;
use crate::router::tree::art::{ArtAllocator, TreeKeyTrait};
use crate::router::tree::art::node::BaseNode;

pub(crate) struct RawTree<K: TreeKeyTrait, V> {
    pub(crate) root: *const Node256,
    allocator: Arc<ArtAllocator>,
    _pt_key: PhantomData<K>,
    _pt_value: PhantomData<V>,
}

unsafe impl<K: TreeKeyTrait, V> Send for RawTree<K, V> {}

unsafe impl<K: TreeKeyTrait, V> Sync for RawTree<K, V> {}

impl<K: TreeKeyTrait, V> Default for RawTree<K, V> {
    fn default() -> Self {
        todo!()
    }
}

impl<K: TreeKeyTrait, V> Drop for RawTree<K, V> {
    fn drop(&mut self) {
        todo!()
    }
}

impl<K: TreeKeyTrait, V> RawTree<K, V> {
    pub fn new(allocator: Arc<ArtAllocator>) -> Self {
        RawTree {
            root: BaseNode::make_node::<Node256>(&[], allocator.clone())
                .expect("Can't allocate memory for root node!") as *const Node256,
            allocator,
            _pt_key: PhantomData,
            _pt_value: PhantomData,
        }
    }
}

impl<T: RawKey,V> RawTree<T, V> {

}
