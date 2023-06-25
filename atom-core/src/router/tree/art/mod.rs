use std::alloc::{Allocator, AllocError, Layout};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Arc;
// use crate::router::tree::art::tree::RawTree;

mod tree;
pub mod node;
pub mod utils;
pub mod guard;

/// readme
/// https://zhuanlan.zhihu.com/p/142407392 实现相关文件
/// 并发算法 Multi-ART
/// https://zhuanlan.zhihu.com/p/65414186
/// 并发优化
/// https://oreki.blog.csdn.net/article/details/130119444?spm=1001.2101.3001.6650.1&utm_medium=distribute.pc_relevant.none-task-blog-2%7Edefault%7ECTRLIST%7ERate-1-130119444-blog-119805160.235%5Ev38%5Epc_relevant_sort_base1&depth_1-utm_source=distribute.pc_relevant.none-task-blog-2%7Edefault%7ECTRLIST%7ERate-1-130119444-blog-119805160.235%5Ev38%5Epc_relevant_sort_base1&utm_relevant_index=2
#[derive(Clone)]
pub struct ArtAllocator;

unsafe impl Sync for ArtAllocator {}
unsafe impl Send for ArtAllocator {}


pub fn get_art_allocator() -> Arc<ArtAllocator> {
    return Arc::new(ArtAllocator {});
}

unsafe impl Allocator for ArtAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = unsafe { std::alloc::alloc(layout) };
        let ptr_slice = std::ptr::slice_from_raw_parts_mut(ptr, layout.size());
        Ok(NonNull::new(ptr_slice).unwrap())
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        std::alloc::dealloc(ptr.as_ptr(), layout);
    }
}
pub trait TreeKeyTrait {
    fn len(&self) -> usize;

    fn as_bytes(&self) -> &[u8];
}

impl TreeKeyTrait for String {
    fn len(&self) -> usize {
        self.len()
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}
pub(crate) struct ArtTree<K,V> where K: TreeKeyTrait {
    // inner: RawTree<K,V>,
    pr_key: PhantomData<K>,
    pr_value: PhantomData<V>
}

impl<K: TreeKeyTrait, V> Default for ArtTree<K, V> {
    fn default() -> Self {
       todo!()
    }
}

