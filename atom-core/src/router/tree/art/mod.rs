use std::alloc::{Allocator, AllocError, Layout};
use std::marker::PhantomData;
use std::ptr::NonNull;
use crate::router::tree::art::tree::RawTree;

mod tree;
mod node;

#[derive(Clone)]
pub(crate) struct ArtAllocator;

unsafe impl Sync for ArtAllocator {}
unsafe impl Send for ArtAllocator {}

unsafe impl Allocator for ArtAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        // 内存对齐进行内存分配
        todo!()
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        todo!()
    }
}
pub(crate) trait ArtKey: Default {
    fn get_bytes(&self) -> &[u8];
    fn get_mut_bytes(&mut self) -> &mut [u8];
}
pub(crate) struct ArtTree<K,V> where K: ArtKey {
    inner: RawTree<K,V>,
    pr_key: PhantomData<K>,
    pr_value: PhantomData<V>
}

impl<K: ArtKey, V> Default for ArtTree<K, V> {
    fn default() -> Self {
       todo!()
    }
}

