use std::alloc::{Layout, LayoutError};
use std::mem;
use crate::router::tree::art::node::{BaseNode, NodeTrait, NodeType};

pub(crate) const NODE4TYPE: Result<Layout, LayoutError> = Layout::from_size_align(mem::size_of::<Node4>(),
                                                                                  mem::align_of::<Node4>(), );
pub(crate) const NODE16TYPE: Result<Layout, LayoutError> = Layout::from_size_align(mem::size_of::<Node16>(),
                                                                                   mem::align_of::<Node16>(), );
pub(crate) const NODE48TYPE: Result<Layout, LayoutError> = Layout::from_size_align(mem::size_of::<Node48>(),
                                                                                   mem::align_of::<Node48>(), );
pub(crate) const NODE256TYPE: Result<Layout, LayoutError> = Layout::from_size_align(mem::size_of::<Node256>(),
                                                                                    mem::align_of::<Node256>(), );

pub const SMALL_STRUCT: usize = 8;

pub type Small = [u8; SMALL_STRUCT];

#[cfg(all(target_feature = "sse2", not(miri)))]
use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
use std::arch::x86_64::{
    __m128i, _mm_cmplt_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use crate::router::tree::art::node::node_ptr::NodePtr;
use crate::router::tree::art::TreeKeyTrait;

#[repr(C)]
#[repr(align(64))] // 常见的缓存行大小为64字节
pub(crate) struct Node4 {
    base: BaseNode,
    keys: [u8; 4],
    children: [NodePtr; 4],
}

impl NodeTrait for Node4 {
    fn base(&self) -> &BaseNode {
       &self.base
    }

    fn base_mut(&mut self) -> &mut BaseNode {
        &mut self.base
    }

    fn is_full(&self) -> bool {
        self.base.meta.count == 4
    }

    fn insert(&mut self, key: u8, node: NodePtr) {
        let mut pos: usize = 0;

        while (pos as u16) < self.base.meta.count {
            // todo 二分查找， 默认有序 从小到大
            if self.keys[pos] < key {
                pos += 1;
                continue;
            } else {
                break;
            }
        }

        unsafe {
            std::ptr::copy(
                self.keys.as_ptr().add(pos),
                self.keys.as_mut_ptr().add(pos + 1),
                self.base.meta.count as usize - pos,
            );

            std::ptr::copy(
                self.children.as_ptr().add(pos),
                self.children.as_mut_ptr().add(pos + 1),
                self.base.meta.count as usize - pos,
            );
        }

        self.keys[pos] = key;
        self.children[pos] = node;
        self.base.meta.count += 1;
    }

    fn change(&mut self, key: u8, val: NodePtr) -> NodePtr {
        for i in 0..self.base.meta.count {
            // 二分查找
            if self.keys[i as usize] == key {
                let old = self.children[i as usize];
                self.children[i as usize] = val;
                return old;
            }
        }
        unreachable!("The key should always exist in the node");
    }

    fn get_child(&self, key: u8) -> Option<NodeLeaf<K, V>> {
        for i in 0..self.base.meta.count {
            if self.keys[i as usize] == key {
                let child = self.children[i as usize].clone();
                return Some(child);
            }
        }
        None
    }

    fn remove(&mut self, k: u8) {
        for i in 0..self.base.meta.count {
            if self.keys[i as usize] == k {
                unsafe {
                    std::ptr::copy(
                        self.keys.as_ptr().add(i as usize + 1),
                        self.keys.as_mut_ptr().add(i as usize),
                        (self.base.meta.count - i - 1) as usize,
                    );

                    std::ptr::copy(
                        self.children.as_ptr().add(i as usize + 1),
                        self.children.as_mut_ptr().add(i as usize),
                        (self.base.meta.count - i - 1) as usize,
                    )
                }
                self.base.meta.count -= 1;
                return;
            }
        }
    }

    fn copy_to<N: NodeTrait<K,V>>(&self, dst: &mut N) {
        for i in 0..self.base.meta.count {
            dst.insert(self.keys[i as usize], self.children[i as usize];
        }
    }

    fn get_type() -> NodeType {
        NodeType::Node4
    }
}

// pub(crate) struct Node4Iter<'a> {
//     start: u8,
//     end: u8,
//     idx: u8,
//     cnt: u8,
//     node: &'a Node4,
// }
//
// impl Iterator for Node4Iter<'_> {
//     type Item = (u8, NodePtr);
//
//     fn next(&mut self) -> Option<Self::Item> {
//         loop {
//             if self.idx >= self.cnt {
//                 return None;
//             }
//             let cur = self.idx;
//             self.idx += 1;
//
//             let key = self.node.keys[cur as usize];
//             if key >= self.start && key <= self.end {
//                 return Some((key, self.node.children[cur as usize]));
//             }
//         }
//     }
// }

#[repr(C)]
#[repr(align(8))]
pub(crate) struct Node16<K:TreeKeyTrait, V> {
    base: BaseNode,
    children: [NodeLeaf<K,V>; 16],
    keys: [u8; 16],
}
//
// impl Node16 {
//     /// 翻转 u8 的符号位
//     fn flip_sign(val: u8) -> u8 {
//         val ^ 128
//     }
//
//     #[cfg(all(target_feature = "sse2", not(miri)))]
//     fn ctz(val: u16) -> u16 {
//         val.trailing_zeros() as u16
//     }
//
//     fn get_insert_pos(&self, key: u8) -> usize {
//         let flipped = Self::flip_sign(key);
//
//         #[cfg(all(target_feature = "sse2", not(miri)))]
//         {
//             unsafe {
//
//                 // __m128i 是一个宽度为 128 位的整数向量类型。它可以存储和操作 16 个 8 位整数值。
//
//                 // _mm_cmplt_epi8: 这是一个 SSE 指令，用于比较两个 __m128i 向量中对应元素的大小。
//                 // 它返回一个 __m128i 向量，每个元素的值为 0 或 1，表示对应位置的元素是否满足小于的条件。
//                 let cmp = _mm_cmplt_epi8(
//                     _mm_set1_epi8(flipped as i8), // 用于将一个 8 位整数值复制到一个 __m128i 向量的每个元素中。
//                     _mm_loadu_si128(&self.keys as *const [u8; 16] as *const __m128i), // 用于从内存中加载 128 位的整数向量到一个 __m128i 变量中。它可以处理未对齐的内存地址。
//                 );
//                 // _mm_movemask_epi8，用于将 __m128i 向量中的元素转换为位掩码。它返回一个 16 位整数，其中每个位表示对应元素的高位是否为 1。
//                 let bit_field = _mm_movemask_epi8(cmp) & (0xFFFF >> (16 - self.base.meta.count)); // 将超过 self.base.meta.count 位的高位清零，以确保只考虑有效元素。
//                 let pos = if bit_field > 0 {
//                     Self::ctz(bit_field as u16)
//                 } else {
//                     self.base.meta.count
//                 };
//                 pos as usize
//             }
//         }
//
//         #[cfg(any(not(target_feature = "sse2"), miri))]
//         {
//             let mut pos = 0;
//             while pos < self.base.meta.count {
//                 if self.keys[pos as usize] >= flipped {
//                     return pos as usize;
//                 }
//                 pos += 1;
//             }
//             pos as usize
//         }
//     }
//
//     fn get_child_pos(&self, key: u8) -> Option<usize> {
//         #[cfg(all(target_feature = "sse2", not(miri)))]
//         unsafe {
//             self.get_child_pos_sse2(key)
//         }
//
//         #[cfg(any(not(target_feature = "sse2"), miri))]
//         self.get_child_pos_linear(key)
//     }
//
//     #[cfg(any(not(target_feature = "sse2"), miri))]
//     fn get_child_pos_linear(&self, key: u8) -> Option<usize> {
//         for i in 0..self.base.meta.count {
//             if self.keys[i as usize] == Self::flip_sign(key) {
//                 return Some(i as usize);
//             }
//         }
//         None
//     }
//
//     #[cfg(target_feature = "sse2")]
//     unsafe fn get_child_pos_sse2(&self, key: u8) -> Option<usize> {
//         use std::arch::x86_64::{
//             __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
//         };
//         let cmp = _mm_cmpeq_epi8(
//             _mm_set1_epi8(Self::flip_sign(key) as i8),
//             _mm_loadu_si128(&self.keys as *const [u8; 16] as *const __m128i),
//         );
//         let bit_field = _mm_movemask_epi8(cmp) & ((1 << self.base.meta.count) - 1);
//         if bit_field > 0 {
//             Some(Self::ctz(bit_field as u16) as usize)
//         } else {
//             None
//         }
//     }
// }
//
// impl NodeTrait for Node16 {
//     fn get_type() -> NodeType {
//         NodeType::Node16
//     }
//
//     // fn get_children(&self, start: u8, end: u8) -> NodeIter {
//     //     if self.base.meta.count == 0 {
//     //         // FIXME: the node may be empty due to deletion, this is not intended, we should fix the delete logic
//     //         return NodeIter::N16(Node16Iter {
//     //             node: self,
//     //             start_pos: 1,
//     //             end_pos: 0,
//     //         });
//     //     }
//     //     let start_pos = self.get_child_pos(start).unwrap_or(0);
//     //     let end_pos = self
//     //         .get_child_pos(end)
//     //         .unwrap_or(self.base.meta.count as usize - 1);
//     //
//     //     debug_assert!(end_pos < 16);
//     //
//     //     NodeIter::N16(Node16Iter {
//     //         node: self,
//     //         start_pos,
//     //         end_pos,
//     //     })
//     // }
//
//     fn remove(&mut self, k: u8) {
//         let pos = self
//             .get_child_pos(k)
//             .expect("trying to delete a non-existing key");
//         unsafe {
//             std::ptr::copy(
//                 self.keys.as_ptr().add(pos + 1),
//                 self.keys.as_mut_ptr().add(pos),
//                 self.base.meta.count as usize - pos - 1,
//             );
//
//             std::ptr::copy(
//                 self.children.as_ptr().add(pos + 1),
//                 self.children.as_mut_ptr().add(pos),
//                 self.base.meta.count as usize - pos - 1,
//             );
//         }
//         self.base.meta.count -= 1;
//         debug_assert!(self.get_child(k).is_none());
//     }
//
//     fn copy_to<N: NodeTrait>(&self, dst: &mut N) {
//         for i in 0..self.base.meta.count {
//             dst.insert(
//                 Self::flip_sign(self.keys[i as usize]),
//                 self.children[i as usize],
//             );
//         }
//     }
//
//     fn base(&self) -> &BaseNode {
//         &self.base
//     }
//
//     fn base_mut(&mut self) -> &mut BaseNode {
//         &mut self.base
//     }
//
//     fn is_full(&self) -> bool {
//         self.base.meta.count == 16
//     }
//
//     // Insert must keep keys sorted, is this necessary?
//     fn insert(&mut self, key: u8, node: NodePtr) {
//         let key_flipped = Self::flip_sign(key);
//
//         let pos = self.get_insert_pos(key);
//
//         unsafe {
//             std::ptr::copy(
//                 self.keys.as_ptr().add(pos),
//                 self.keys.as_mut_ptr().add(pos + 1),
//                 self.base.meta.count as usize - pos,
//             );
//
//             std::ptr::copy(
//                 self.children.as_ptr().add(pos),
//                 self.children.as_mut_ptr().add(pos + 1),
//                 self.base.meta.count as usize - pos,
//             );
//         }
//
//         self.keys[pos] = key_flipped;
//         self.children[pos] = node;
//         self.base.meta.count += 1;
//
//         assert!(self.base.meta.count <= 16);
//     }
//
//     fn change(&mut self, key: u8, val: NodePtr) -> NodePtr {
//         let pos = self.get_child_pos(key).unwrap();
//         let old = self.children[pos];
//         self.children[pos] = val;
//         old
//     }
//
//     fn get_child(&self, key: u8) -> Option<NodePtr> {
//         let pos = self.get_child_pos(key)?;
//         let child = self.children[pos];
//         Some(child)
//     }
// }
//
#[repr(C)]
#[repr(align(8))]
pub(crate) struct Node48<K,V> {
    base: BaseNode,
    pub(crate) child_idx: [u8; 256],
    next_empty: u8,
    children: [NodeLeaf<K,V>; 48],
}

// pub(crate) const EMPTY_MARKER: u8 = 48;
//
// impl Node48 {
//     pub(crate) fn init_empty(&mut self) {
//         for v in self.child_idx.iter_mut() {
//             *v = EMPTY_MARKER;
//         }
//         self.next_empty = 0;
//         for (i, child) in self.children.iter_mut().enumerate() {
//             *child = NodePtr::from_tid(i + 1);
//         }
//     }
// }
//
// impl NodeTrait for Node48 {
//     fn get_type() -> NodeType {
//         NodeType::Node48
//     }
//
//     fn remove(&mut self, k: u8) {
//         debug_assert!(self.child_idx[k as usize] != EMPTY_MARKER);
//         let pos = self.child_idx[k as usize];
//         self.children[pos as usize] = NodePtr::from_tid(self.next_empty as usize);
//         self.child_idx[k as usize] = EMPTY_MARKER;
//         self.next_empty = pos;
//         self.base.meta.count -= 1;
//         debug_assert!(self.get_child(k).is_none());
//     }
//
//     // fn get_children(&self, start: u8, end: u8) -> NodeIter {
//     //     NodeIter::N48(Node48Iter {
//     //         start: start as u16,
//     //         end: end as u16,
//     //         node: self,
//     //     })
//     // }
//
//     fn copy_to<N: NodeTrait>(&self, dst: &mut N) {
//         for (i, c) in self.child_idx.iter().enumerate() {
//             if *c != EMPTY_MARKER {
//                 dst.insert(i as u8, self.children[*c as usize]);
//             }
//         }
//     }
//
//     fn base(&self) -> &BaseNode {
//         &self.base
//     }
//
//     fn base_mut(&mut self) -> &mut BaseNode {
//         &mut self.base
//     }
//
//     fn is_full(&self) -> bool {
//         self.base.meta.count == 48
//     }
//
//     fn insert(&mut self, key: u8, node: NodePtr) {
//         let pos = self.next_empty as usize;
//         self.next_empty = self.children[pos].as_tid() as u8;
//
//         debug_assert!(pos < 48);
//
//         self.children[pos] = node;
//         self.child_idx[key as usize] = pos as u8;
//         self.base.meta.count += 1;
//     }
//
//     fn change(&mut self, key: u8, val: NodePtr) -> NodePtr {
//         let old = self.children[self.child_idx[key as usize] as usize];
//         self.children[self.child_idx[key as usize] as usize] = val;
//         old
//     }
//
//     fn get_child(&self, key: u8) -> Option<NodePtr> {
//         if self.child_idx[key as usize] == EMPTY_MARKER {
//             None
//         } else {
//             let child = self.children[self.child_idx[key as usize] as usize];
//
//             #[cfg(all(target_feature = "sse2", not(miri)))]
//             {
//                 let ptr = child.as_ptr();
//                 unsafe {
//                     _mm_prefetch::<_MM_HINT_T0>(ptr as *const i8);
//                 }
//             }
//
//             Some(child)
//         }
//     }
// }


#[repr(C)]
#[repr(align(8))]
pub(crate) struct Node256<K: TreeKeyTrait, V> {
    base: BaseNode,
    key_mask: [u8; 32],
    children: [NodeLeaf<K,V>; 256],
}

// impl Node256 {
//     #[inline]
//     fn set_mask(&mut self, key: usize) {
//         let idx = key / 8;
//         let bit = key % 8;
//         self.key_mask[idx] |= 1 << bit;
//     }
//
//     #[inline]
//     fn unset_mask(&mut self, key: usize) {
//         let idx = key / 8;
//         let bit = key % 8;
//         self.key_mask[idx] &= !(1 << bit);
//     }
//
//     #[inline]
//     fn get_mask(&self, key: usize) -> bool {
//         let idx = key / 8;
//         let bit = key % 8;
//         let key_mask = self.key_mask[idx];
//         key_mask & (1 << bit) != 0
//     }
// }
//
// impl NodeTrait for Node256 {
//     fn base(&self) -> &BaseNode {
//         &self.base
//     }
//
//     // fn get_children(&self, start: u8, end: u8) -> NodeIter {
//     //     NodeIter::N256(Node256Iter {
//     //         start,
//     //         end,
//     //         idx: 0,
//     //         node: self,
//     //     })
//     // }
//
//     fn base_mut(&mut self) -> &mut BaseNode {
//         &mut self.base
//     }
//
//     fn is_full(&self) -> bool {
//         false
//     }
//
//     fn insert(&mut self, key: u8, node: NodePtr) {
//         self.children[key as usize] = node;
//         self.set_mask(key as usize);
//         self.base.meta.count += 1;
//     }
//
//     fn change(&mut self, key: u8, val: NodePtr) -> NodePtr {
//         let old = self.children[key as usize];
//         self.children[key as usize] = val;
//         old
//     }
//
//     fn get_child(&self, key: u8) -> Option<NodePtr> {
//         if self.get_mask(key as usize) {
//             let child = self.children[key as usize];
//
//             #[cfg(all(target_feature = "sse2", not(miri)))]
//             {
//                 let ptr = child.as_ptr();
//                 unsafe {
//                     _mm_prefetch::<_MM_HINT_T0>(ptr as *const i8);
//                 }
//             }
//
//             Some(child)
//         } else {
//             None
//         }
//     }
//
//     fn remove(&mut self, k: u8) {
//         self.unset_mask(k as usize);
//         self.base.meta.count -= 1;
//     }
//
//     fn copy_to<N: NodeTrait>(&self, dst: &mut N) {
//         for (i, c) in self.children.iter().enumerate() {
//             if self.get_mask(i) {
//                 dst.insert(i as u8, *c);
//             }
//         }
//     }
//
//     fn get_type() -> NodeType {
//         NodeType::Node256
//     }
// }


pub struct SmallStruct<T> {
    storage: MaybeUninit<Small>,
    marker: PhantomData<T>,
}

// impl<T> Clone for SmallStruct<T> {
//     fn clone(&self) -> Self {
//
//        Self {
//            storage: self.storage.clone_from(),
//            marker: PhantomData
//        }
//     }
// }

pub enum NodeLeaf<K, V> {
    Empty,
    LeafLarge(Box<(K, V)>),
    LeafLargeKey(Box<K>, SmallStruct<V>),
    LeafLargeValue(SmallStruct<K>, Box<V>),
    LeafSmall(SmallStruct<K>, SmallStruct<V>),
}


impl<T> SmallStruct<T> {
    pub fn new(elem: T) -> Self {
        unsafe {
            let mut ret = SmallStruct { storage: MaybeUninit::<Small>::uninit(), marker: PhantomData };
            ret.storage.as_mut_ptr().write(elem);
            ret
        }
    }

    pub fn reference(&self) -> &T {
        unsafe { &*(self.storage.assume_init().as_ptr() as *const T) }
    }

    pub fn own(self) -> T {
        unsafe {
            let mut ret = MaybeUninit::<Small>::uninit();
            let dst = &mut ret as *mut T as *mut u8;
            std::ptr::copy_nonoverlapping(self.storage.assume_init().as_ptr(), dst, mem::size_of::<T>());
            ret
        }
    }
}

impl<K: TreeKeyTrait, V> NodeLeaf<K, V> {
    #[inline]
    pub fn key(&self) -> &K {
        match self {
            &NodeLeaf::LeafLarge(ref ptr) => &ptr.as_ref().0,
            &NodeLeaf::LeafLargeKey(ref key_ptr, _) => &*key_ptr,
            &NodeLeaf::LeafLargeValue(ref key_small, _) => key_small.reference(),
            &NodeLeaf::LeafSmall(ref key_small, _) => key_small.reference(),
            _ => {}
        }
    }

    pub fn value(self) -> V {
        match self {
            NodeLeaf::LeafLarge(ptr) => (*ptr).1,
            NodeLeaf::LeafLargeKey(_, value_small) => value_small.own(),
            NodeLeaf::LeafLargeValue(_, value_ptr) => *value_ptr,
            NodeLeaf::LeafSmall(_, value_small) => value_small.own(),
            _ => panic!("Does not contain value"),
        }
    }

    #[inline]
    pub fn new_leaf(key: K, value: V) -> NodeLeaf<K, V> {
        if mem::size_of::<K>() > SMALL_STRUCT {
            if mem::size_of::<V>() > SMALL_STRUCT {
                NodeLeaf::LeafLarge(Box::new((key, value)))
            } else {
                NodeLeaf::LeafLargeKey(Box::new(key), SmallStruct::new(value))
            }
        } else {
            if mem::size_of::<V>() > SMALL_STRUCT {
                NodeLeaf::LeafLargeValue(SmallStruct::new(key), Box::new(value))
            } else {
                NodeLeaf::LeafSmall(SmallStruct::new(key), SmallStruct::new(value))
            }
        }
    }
}