// use std::alloc::{Layout, LayoutError};
// use std::mem;
// use crate::router::tree::art::node::{BaseNode, NodeTrait, NodeType};
//
// #[cfg(all(target_feature = "sse2", not(miri)))]
// use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
// use std::arch::x86_64::{
//     __m128i, _mm_cmplt_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
// };
// use std::marker::PhantomData;
// use std::mem::MaybeUninit;
// use crate::router::tree::art::node::leaf::NodeLeaf;
// use crate::router::tree::art::TreeKeyTrait;
//
// #[repr(C)]
// #[repr(align(64))] // 常见的缓存行大小为64字节
// pub(crate) struct Node4<K, V> {
//     base: BaseNode,
//     keys: [u8; 4],
//     children: [NodeLeaf<K, V>; 4],
// }
//
// impl<K, V> NodeTrait for Node4<K, V> {
//
//     type Key = K;
//     type Value = V;
//     fn get_type() -> NodeType {
//         NodeType::Node4
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
//         self.base.meta.count == 4
//     }
//
//     fn insert(&mut self, key: u8, node: NodeLeaf<Self::Key, Self::Value>) {
//         let mut pos: usize = 0;
//
//         while (pos as u16) < self.base.meta.count {
//             // todo 二分查找， 默认有序 从小到大
//             if self.keys[pos] < key {
//                 pos += 1;
//                 continue;
//             } else {
//                 break;
//             }
//         }
//         // 判断 插入是否是尾插
//         if self.base.meta.count as usize != pos {
//             unsafe {
//                 std::ptr::copy_nonoverlapping(
//                     self.keys.as_ptr().add(pos),
//                     self.keys.as_mut_ptr().add(pos + 1),
//                     self.base.meta.count as usize - pos,
//                 );
//
//                 std::ptr::copy_nonoverlapping(
//                     self.children.as_ptr().add(pos),
//                     self.children.as_mut_ptr().add(pos + 1),
//                     self.base.meta.count as usize - pos,
//                 );
//             }
//         }
//
//         self.keys[pos] = key;
//         self.children[pos] = node;
//         self.base.meta.count += 1;
//     }
//
//      fn change(&mut self, key: u8, val: NodeLeaf<Self::Key, Self::Value>) -> Option<NodeLeaf<Self::Key, Self::Value>> {
//         for i in 0..self.base.meta.count {
//             // 二分查找
//             if self.keys[i as usize] == key {
//                 unsafe {
//                     let old = mem::replace(&mut self.children[i as usize], val);
//                     return Some(old);
//                 }
//             }
//         }
//         None
//     }
//
//      fn get_child(&self, key: u8) -> Option<&NodeLeaf<Self::Key, Self::Value>> {
//
//              for i in 0..self.base.meta.count {
//                unsafe {
//                    if self.keys[i as usize] == key {
//                        let child = &self.children[i as usize];
//                        return Some(child);
//                    }
//                }
//              }
//              None
//
//      }
//
//     // fn copy_to<N: NodeTrait<K, V>>(&self, dst: &mut N) {
//     //     for i in 0..self.base.meta.count {
//     //         dst.insert(self.keys[i as usize], self.children[i as usize];
//     //     }
//     // }
//
//     fn remove(&mut self, k: u8) {
//         for i in 0..self.base.meta.count {
//             if self.keys[i as usize] == k {
//                 unsafe {
//                     std::ptr::copy(
//                         self.keys.as_ptr().add(i as usize + 1),
//                         self.keys.as_mut_ptr().add(i as usize),
//                         (self.base.meta.count - i - 1) as usize,
//                     );
//
//                     std::ptr::copy(
//                         self.children.as_ptr().add(i as usize + 1),
//                         self.children.as_mut_ptr().add(i as usize),
//                         (self.base.meta.count - i - 1) as usize,
//                     )
//                 }
//                 self.base.meta.count -= 1;
//                 return;
//             }
//         }
//     }
// }
//
// // // pub(crate) struct Node4Iter<'a> {
// // //     start: u8,
// // //     end: u8,
// // //     idx: u8,
// // //     cnt: u8,
// // //     node: &'a Node4,
// // // }
// // //
// // // impl Iterator for Node4Iter<'_> {
// // //     type Item = (u8, NodePtr);
// // //
// // //     fn next(&mut self) -> Option<Self::Item> {
// // //         loop {
// // //             if self.idx >= self.cnt {
// // //                 return None;
// // //             }
// // //             let cur = self.idx;
// // //             self.idx += 1;
// // //
// // //             let key = self.node.keys[cur as usize];
// // //             if key >= self.start && key <= self.end {
// // //                 return Some((key, self.node.children[cur as usize]));
// // //             }
// // //         }
// // //     }
// // // }
// //
// // #[repr(C)]
// // #[repr(align(8))]
// // pub(crate) struct Node16<K, V> {
// //     base: BaseNode,
// //     children: [NodeLeaf<K, V>; 16],
// //     keys: [u8; 16],
// // }
// //
// //
// // impl<K, V> Node16<K, V> {
// //     /// 翻转 u8 的符号位
// //     fn flip_sign(val: u8) -> u8 {
// //         val ^ 128
// //     }
// //
// //     #[cfg(all(target_feature = "sse2", not(miri)))]
// //     fn ctz(val: u16) -> u16 {
// //         val.trailing_zeros() as u16
// //     }
// //
// //     fn get_insert_pos(&self, key: u8) -> usize {
// //         let flipped = Self::flip_sign(key);
// //
// //         #[cfg(all(target_feature = "sse2", not(miri)))]
// //         {
// //             unsafe {
// //
// //                 // __m128i 是一个宽度为 128 位的整数向量类型。它可以存储和操作 16 个 8 位整数值。
// //
// //                 // _mm_cmplt_epi8: 这是一个 SSE 指令，用于比较两个 __m128i 向量中对应元素的大小。
// //                 // 它返回一个 __m128i 向量，每个元素的值为 0 或 1，表示对应位置的元素是否满足小于的条件。
// //                 let cmp = _mm_cmplt_epi8(
// //                     _mm_set1_epi8(flipped as i8), // 用于将一个 8 位整数值复制到一个 __m128i 向量的每个元素中。
// //                     _mm_loadu_si128(&self.keys as *const [u8; 16] as *const __m128i), // 用于从内存中加载 128 位的整数向量到一个 __m128i 变量中。它可以处理未对齐的内存地址。
// //                 );
// //                 // _mm_movemask_epi8，用于将 __m128i 向量中的元素转换为位掩码。它返回一个 16 位整数，其中每个位表示对应元素的高位是否为 1。
// //                 let bit_field = _mm_movemask_epi8(cmp) & (0xFFFF >> (16 - self.base.meta.count)); // 将超过 self.base.meta.count 位的高位清零，以确保只考虑有效元素。
// //                 let pos = if bit_field > 0 {
// //                     Self::ctz(bit_field as u16)
// //                 } else {
// //                     self.base.meta.count
// //                 };
// //                 pos as usize
// //             }
// //         }
// //
// //         #[cfg(any(not(target_feature = "sse2"), miri))]
// //         {
// //             let mut pos = 0;
// //             while pos < self.base.meta.count {
// //                 if self.keys[pos as usize] >= flipped {
// //                     return pos as usize;
// //                 }
// //                 pos += 1;
// //             }
// //             pos as usize
// //         }
// //     }
// //
// //     fn get_child_pos(&self, key: u8) -> Option<usize> {
// //         #[cfg(all(target_feature = "sse2", not(miri)))]
// //         unsafe {
// //             self.get_child_pos_sse2(key)
// //         }
// //
// //         #[cfg(any(not(target_feature = "sse2"), miri))]
// //         self.get_child_pos_linear(key)
// //     }
// //
// //     #[cfg(any(not(target_feature = "sse2"), miri))]
// //     fn get_child_pos_linear(&self, key: u8) -> Option<usize> {
// //         for i in 0..self.base.meta.count {
// //             if self.keys[i as usize] == Self::flip_sign(key) {
// //                 return Some(i as usize);
// //             }
// //         }
// //         None
// //     }
// //
// //     #[cfg(target_feature = "sse2")]
// //     unsafe fn get_child_pos_sse2(&self, key: u8) -> Option<usize> {
// //         use std::arch::x86_64::{
// //             __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
// //         };
// //         let cmp = _mm_cmpeq_epi8(
// //             _mm_set1_epi8(Self::flip_sign(key) as i8),
// //             _mm_loadu_si128(&self.keys as *const [u8; 16] as *const __m128i),
// //         );
// //         let bit_field = _mm_movemask_epi8(cmp) & ((1 << self.base.meta.count) - 1);
// //         if bit_field > 0 {
// //             Some(Self::ctz(bit_field as u16) as usize)
// //         } else {
// //             None
// //         }
// //     }
// // }
// //
// // // impl<K,V> NodeTrait for Node16<K, V> {
// // //     fn get_type() -> NodeType {
// // //         NodeType::Node16
// // //     }
// // //
// // //     fn base(&self) -> &BaseNode {
// // //         &self.base
// // //     }
// // //
// // //     fn base_mut(&mut self) -> &mut BaseNode {
// // //         &mut self.base
// // //     }
// // //
// // //     fn is_full(&self) -> bool {
// // //         self.base.meta.count == 16
// // //     }
// // //
// // //     fn insert<K: TreeKeyTrait,V>(&mut self, key: u8, node: NodeLeaf<K,V>) {
// // //         let key_flipped = Self::flip_sign(key);
// // //
// // //         let pos = self.get_insert_pos(key);
// // //
// // //         unsafe {
// // //             std::ptr::copy(
// // //                 self.keys.as_ptr().add(pos),
// // //                 self.keys.as_mut_ptr().add(pos + 1),
// // //                 self.base.meta.count as usize - pos,
// // //             );
// // //
// // //             std::ptr::copy(
// // //                 self.children.as_ptr().add(pos),
// // //                 self.children.as_mut_ptr().add(pos + 1),
// // //                 self.base.meta.count as usize - pos,
// // //             );
// // //         }
// // //
// // //         self.keys[pos] = key_flipped;
// // //         self.children[pos] = node;
// // //         self.base.meta.count += 1;
// // //
// // //         assert!(self.base.meta.count <= 16);
// // //     }
// // //
// // //     fn change<K: TreeKeyTrait, V>(&mut self, key: u8, val: NodeLeaf<K, V>) -> NodeLeaf<K, V> {
// // //         todo!()
// // //     }
// // //
// // //     fn get_child<K: TreeKeyTrait, V>(&self, key: u8) -> Option<NodeLeaf<K, V>> {
// // //         todo!()
// // //     }
// // //
// // //     fn remove(&mut self, k: u8) {
// // //         let pos = self
// // //             .get_child_pos(k)
// // //             .expect("trying to delete a non-existing key");
// // //         unsafe {
// // //             std::ptr::copy(
// // //                 self.keys.as_ptr().add(pos + 1),
// // //                 self.keys.as_mut_ptr().add(pos),
// // //                 self.base.meta.count as usize - pos - 1,
// // //             );
// // //
// // //             std::ptr::copy(
// // //                 self.children.as_ptr().add(pos + 1),
// // //                 self.children.as_mut_ptr().add(pos),
// // //                 self.base.meta.count as usize - pos - 1,
// // //             );
// // //         }
// // //         self.base.meta.count -= 1;
// // //         debug_assert!(self.get_child(k).is_none());
// // //     }
// // // }
// //
// // // #[repr(C)]
// // // #[repr(align(8))]
// // // pub(crate) struct Node48 {
// // //     base: BaseNode,
// // //     pub(crate) child_idx: [u8; 256],
// // //     next_empty: u8,
// // //     children: [NodePtr; 48],
// // // }
// //
// // // pub(crate) const EMPTY_MARKER: u8 = 48;
// // //
// // // impl Node48 {
// // //     pub(crate) fn init_empty(&mut self) {
// // //         for v in self.child_idx.iter_mut() {
// // //             *v = EMPTY_MARKER;
// // //         }
// // //         self.next_empty = 0;
// // //         for (i, child) in self.children.iter_mut().enumerate() {
// // //             *child = NodePtr::from_tid(i + 1);
// // //         }
// // //     }
// // // }
// // //
// // // impl NodeTrait for Node48 {
// // //     fn get_type() -> NodeType {
// // //         NodeType::Node48
// // //     }
// // //
// // //     fn remove(&mut self, k: u8) {
// // //         debug_assert!(self.child_idx[k as usize] != EMPTY_MARKER);
// // //         let pos = self.child_idx[k as usize];
// // //         self.children[pos as usize] = NodePtr::from_tid(self.next_empty as usize);
// // //         self.child_idx[k as usize] = EMPTY_MARKER;
// // //         self.next_empty = pos;
// // //         self.base.meta.count -= 1;
// // //         debug_assert!(self.get_child(k).is_none());
// // //     }
// // //
// // //     // fn get_children(&self, start: u8, end: u8) -> NodeIter {
// // //     //     NodeIter::N48(Node48Iter {
// // //     //         start: start as u16,
// // //     //         end: end as u16,
// // //     //         node: self,
// // //     //     })
// // //     // }
// // //
// // //     fn copy_to<N: NodeTrait>(&self, dst: &mut N) {
// // //         for (i, c) in self.child_idx.iter().enumerate() {
// // //             if *c != EMPTY_MARKER {
// // //                 dst.insert(i as u8, self.children[*c as usize]);
// // //             }
// // //         }
// // //     }
// // //
// // //     fn base(&self) -> &BaseNode {
// // //         &self.base
// // //     }
// // //
// // //     fn base_mut(&mut self) -> &mut BaseNode {
// // //         &mut self.base
// // //     }
// // //
// // //     fn is_full(&self) -> bool {
// // //         self.base.meta.count == 48
// // //     }
// // //
// // //     fn insert(&mut self, key: u8, node: NodePtr) {
// // //         let pos = self.next_empty as usize;
// // //         self.next_empty = self.children[pos].as_tid() as u8;
// // //
// // //         debug_assert!(pos < 48);
// // //
// // //         self.children[pos] = node;
// // //         self.child_idx[key as usize] = pos as u8;
// // //         self.base.meta.count += 1;
// // //     }
// // //
// // //     fn change(&mut self, key: u8, val: NodePtr) -> NodePtr {
// // //         let old = self.children[self.child_idx[key as usize] as usize];
// // //         self.children[self.child_idx[key as usize] as usize] = val;
// // //         old
// // //     }
// // //
// // //     fn get_child(&self, key: u8) -> Option<NodePtr> {
// // //         if self.child_idx[key as usize] == EMPTY_MARKER {
// // //             None
// // //         } else {
// // //             let child = self.children[self.child_idx[key as usize] as usize];
// // //
// // //             #[cfg(all(target_feature = "sse2", not(miri)))]
// // //             {
// // //                 let ptr = child.as_ptr();
// // //                 unsafe {
// // //                     _mm_prefetch::<_MM_HINT_T0>(ptr as *const i8);
// // //                 }
// // //             }
// // //
// // //             Some(child)
// // //         }
// // //     }
// // // }
// //
// //
// // // #[repr(C)]
// // // #[repr(align(8))]
// // // pub(crate) struct Node256 {
// // //     base: BaseNode,
// // //     key_mask: [u8; 32],
// // //     children: [NodePtr; 256],
// // // }
// //
// // // impl Node256 {
// // //     #[inline]
// // //     fn set_mask(&mut self, key: usize) {
// // //         let idx = key / 8;
// // //         let bit = key % 8;
// // //         self.key_mask[idx] |= 1 << bit;
// // //     }
// // //
// // //     #[inline]
// // //     fn unset_mask(&mut self, key: usize) {
// // //         let idx = key / 8;
// // //         let bit = key % 8;
// // //         self.key_mask[idx] &= !(1 << bit);
// // //     }
// // //
// // //     #[inline]
// // //     fn get_mask(&self, key: usize) -> bool {
// // //         let idx = key / 8;
// // //         let bit = key % 8;
// // //         let key_mask = self.key_mask[idx];
// // //         key_mask & (1 << bit) != 0
// // //     }
// // // }
// // //
// // // impl NodeTrait for Node256 {
// // //     fn base(&self) -> &BaseNode {
// // //         &self.base
// // //     }
// // //
// // //     // fn get_children(&self, start: u8, end: u8) -> NodeIter {
// // //     //     NodeIter::N256(Node256Iter {
// // //     //         start,
// // //     //         end,
// // //     //         idx: 0,
// // //     //         node: self,
// // //     //     })
// // //     // }
// // //
// // //     fn base_mut(&mut self) -> &mut BaseNode {
// // //         &mut self.base
// // //     }
// // //
// // //     fn is_full(&self) -> bool {
// // //         false
// // //     }
// // //
// // //     fn insert(&mut self, key: u8, node: NodePtr) {
// // //         self.children[key as usize] = node;
// // //         self.set_mask(key as usize);
// // //         self.base.meta.count += 1;
// // //     }
// // //
// // //     fn change(&mut self, key: u8, val: NodePtr) -> NodePtr {
// // //         let old = self.children[key as usize];
// // //         self.children[key as usize] = val;
// // //         old
// // //     }
// // //
// // //     fn get_child(&self, key: u8) -> Option<NodePtr> {
// // //         if self.get_mask(key as usize) {
// // //             let child = self.children[key as usize];
// // //
// // //             #[cfg(all(target_feature = "sse2", not(miri)))]
// // //             {
// // //                 let ptr = child.as_ptr();
// // //                 unsafe {
// // //                     _mm_prefetch::<_MM_HINT_T0>(ptr as *const i8);
// // //                 }
// // //             }
// // //
// // //             Some(child)
// // //         } else {
// // //             None
// // //         }
// // //     }
// // //
// // //     fn remove(&mut self, k: u8) {
// // //         self.unset_mask(k as usize);
// // //         self.base.meta.count -= 1;
// // //     }
// // //
// // //     fn copy_to<N: NodeTrait>(&self, dst: &mut N) {
// // //         for (i, c) in self.children.iter().enumerate() {
// // //             if self.get_mask(i) {
// // //                 dst.insert(i as u8, *c);
// // //             }
// // //         }
// // //     }
// // //
// // //     fn get_type() -> NodeType {
// // //         NodeType::Node256
// // //     }
// // // }
// //
// //
// // pub struct SmallStruct<T> {
// //     storage: MaybeUninit<Small>,
// //     marker: PhantomData<T>,
// // }
// //
// // // impl<T> Clone for SmallStruct<T> {
// // //     fn clone(&self) -> Self {
// // //
// // //        Self {
// // //            storage: self.storage.clone_from(),
// // //            marker: PhantomData
// // //        }
// // //     }
// // // }
// //
//
//
// #[cfg(test)]
// mod test {
//     use crate::router::tree::art::node::NodeMeta;
//     use super::*;
//
//     #[test]
//     fn string_vector() {
//       let mut temp: Node4<String, String> = Node4 {
//           base: BaseNode { type_version: Default::default(), meta: NodeMeta {
//               node_type: NodeType::Node4,
//               node_prefix: [0;8],
//               prefix_size: 0,
//               count: 1,
//           } },
//           keys: [0;4],
//           children: unsafe { mem::uninitialized() },
//       };
//         assert_eq!(temp.base().meta.node_type, NodeType::Node4);
//         let a = String::from("key1");
//         let b = String::from("value1");
//         let c = String::from("key2");
//         let d = String::from("value2");
//         let node1 = NodeLeaf::<String, String>::new_leaf(a,b);
//         temp.insert(0, node1);
//         assert_eq!(temp.get_child(0).is_some(),true);
//         assert_eq!(temp.get_child(1).is_some(),false);
//
//     }
//     #[test]
//     fn test_u8() {
//         let mut temp: Node4<u8, u8> = Node4 {
//             base: BaseNode { type_version: Default::default(), meta: NodeMeta {
//                 node_type: NodeType::Node4,
//                 node_prefix: [0;8],
//                 prefix_size: 0,
//                 count: 1,
//             } },
//             keys: [0;4],
//             children: unsafe { mem::uninitialized() },
//         };
//
//         assert_eq!(temp.base().meta.node_type, NodeType::Node4);
//         let node1 = NodeLeaf::<u8, u8>::new_leaf(1,1);
//         temp.insert(0, node1);
//         assert_eq!(temp.get_child(0).is_some(),true);
//         assert_eq!(temp.get_child(1).is_some(),false);
//         assert_eq!(temp.get_child(0).expect("ww").key(),Some(1u8));
//     }
// }