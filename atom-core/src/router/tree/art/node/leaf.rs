// use std::marker::PhantomData;
// use std::{mem, ptr};
// use std::mem::MaybeUninit;
//
// pub const SMALL_STRUCT: usize = 16;
//
// pub type Small = [u8; SMALL_STRUCT];
//
// pub enum NodeLeaf<K, V> {
//     Empty,
//     LeafLarge(Box<(K, V)>),
//     LeafLargeKey(Box<K>, SmallStruct<V>),
//     LeafLargeValue(SmallStruct<K>, Box<V>),
//     LeafSmall(SmallStruct<K>, SmallStruct<V>),
// }
//
// pub struct SmallStruct<T> {
//     storage: MaybeUninit<T>,
// }
//
// // impl<T> Drop for SmallStruct<T> {
// //     fn drop(&mut self) {
// //         unsafe {
// //             self.storage.assume_init_drop()
// //         }
// //     }
// // }
//
// impl<T> SmallStruct<T> {
//     pub fn new(elem: T) -> Self {
//         unsafe {
//             let mut ret = SmallStruct { storage: MaybeUninit::<T>::uninit() };
//             ret.storage.write(elem);
//             ret
//         }
//     }
//
//     pub fn reference(&self) -> &T {
//         unsafe { self.storage.assume_init_ref() }
//     }
//
//     pub fn own(self) -> T {
//         unsafe {
//             self.storage.assume_init()
//         }
//     }
// }
//
// impl<K, V> NodeLeaf<K, V> {
//     #[inline]
//     pub fn key(&self) -> Option<&K> {
//         match self {
//             &NodeLeaf::LeafLarge(ref ptr) => Some(&ptr.as_ref().0),
//             &NodeLeaf::LeafLargeKey(ref key_ptr, _) => Some(&*key_ptr),
//             &NodeLeaf::LeafLargeValue(ref key_small, _) => Some(key_small.reference()),
//             &NodeLeaf::LeafSmall(ref key_small, _) => Some(key_small.reference()),
//             _ => {
//                 None
//             }
//         }
//     }
//
//     pub fn value(self) -> V {
//         match self {
//             NodeLeaf::LeafLarge(ptr) => (*ptr).1,
//             NodeLeaf::LeafLargeKey(_, value_small) => value_small.own(),
//             NodeLeaf::LeafLargeValue(_, value_ptr) => *value_ptr,
//             NodeLeaf::LeafSmall(_, value_small) => value_small.own(),
//             _ => panic!("Does not contain value"),
//         }
//     }
//
//     #[inline]
//     pub fn new_leaf(key: K, value: V) -> NodeLeaf<K, V> {
//         if mem::size_of::<K>() > SMALL_STRUCT {
//             if mem::size_of::<V>() > SMALL_STRUCT {
//                 NodeLeaf::LeafLarge(Box::new((key, value)))
//             } else {
//                 NodeLeaf::LeafLargeKey(Box::new(key), SmallStruct::new(value))
//             }
//         } else {
//             if mem::size_of::<V>() > SMALL_STRUCT {
//                 NodeLeaf::LeafLargeValue(SmallStruct::new(key), Box::new(value))
//             } else {
//                 NodeLeaf::LeafSmall(SmallStruct::new(key), SmallStruct::new(value))
//             }
//         }
//     }
// }
//
//
// mod test {
//     use super::*;
//
//     #[test]
//     fn new_leaf_node() {
//         let node = [9; 512];
//         let key = "111";
//         let a = NodeLeaf::new_leaf(key, node);
//         assert_eq!(a.key(), Some(&key));
//         assert_eq!(a.value(), node);
//         let a = NodeLeaf::new_leaf([9; 512], node);
//         // assert_eq!(a.key(), Some(&"111".to_string()));
//         assert_eq!(a.value(), node);
//     }
// }