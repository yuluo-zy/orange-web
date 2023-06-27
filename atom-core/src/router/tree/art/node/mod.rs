use std::sync::atomic::AtomicUsize;
use crate::router::tree::art::node::bit_set::{Bitset16, Bitset64, Bitset8};
use crate::router::tree::art::node::direct_node::DirectNode;
use crate::router::tree::art::node::index_node::IndexNode;
use crate::router::tree::art::node::node::KeyedNode;
use crate::router::tree::art::node::partials::Partial;

pub mod keys;
pub mod partials;
pub mod bit_set;
pub mod bit_array;
pub mod node;
pub mod direct_node;
pub mod index_node;

pub(crate) const MAX_KEY_LEN: usize = 8;

pub(crate) type Prefix = [u8; MAX_KEY_LEN];


pub(crate) struct NodeMeta {
    prefix_cnt: u32,
    pub(crate) count: u16,
    prefix: Prefix,
}


pub trait NodeTrait<N, const NUM_CHILDREN: usize> {
    fn add_child(&mut self, key: u8, node: N);
    fn update_child(&mut self, key: u8, node: N);
    fn find_child(&self, key: u8) -> Option<&N>;
    fn find_child_mut(&mut self, key: u8) -> Option<&mut N>;
    fn delete_child(&mut self, key: u8) -> Option<N>;
    fn num_children(&self) -> usize;
    fn width(&self) -> usize { NUM_CHILDREN }
}

pub(crate) struct Node<P: Partial + Clone, V> {
    pub(crate) prefix: P,
    pub(crate) type_version_lock_obsolete: AtomicUsize,
    pub(crate) meta: NodeMeta,
    pub(crate) ntype: NodeType<P, V>,
}

pub struct NodeLeaf<P: Partial + Clone, V> {
    kay: P,
    value: Option<V>,
}

impl<P: Partial + Clone, V> NodeLeaf<P, V> {
    pub fn value(&self) -> Option<&V> {
        self.value.as_ref()
    }

    pub fn value_mut(&mut self) -> Option<&mut V> {
        self.value.as_mut()
    }
}

pub(crate) enum NodeType<P: Partial + Clone, V> {
    Leaf(NodeLeaf<P, V>),
    Node4(KeyedNode<NodeLeaf<P, V>, 4, Bitset8<1>>),
    Node16(KeyedNode<NodeLeaf<P, V>, 16, Bitset16<1>>),
    Node48(IndexNode<NodeLeaf<P, V>, 48, Bitset64<1>>),
    Node256(DirectNode<NodeLeaf<P, V>>),
}

// impl<P: Partial + Clone, V> Node<P, V> {
//     #[inline]
//     pub(crate) fn new_leaf(key: P, value: V) -> Node<P, V> {
//         Self {
//             prefix: key,
//             ntype: NodeType::Leaf(value),
//         }
//     }
//
//     #[inline]
//     pub fn new_inner(prefix: P) -> Self {
//         let nt = NodeType::Node4(KeyedNode::new());
//         Self { prefix, ntype: nt }
//     }
//
//     #[inline]
//     #[allow(dead_code)]
//     pub fn new_4(prefix: P) -> Self {
//         let nt = NodeType::Node4(KeyedNode::new());
//         Self { prefix, ntype: nt }
//     }
//
//     #[inline]
//     #[allow(dead_code)]
//     pub fn new_16(prefix: P) -> Self {
//         let nt = NodeType::Node16(KeyedNode::new());
//         Self { prefix, ntype: nt }
//     }
//
//     #[inline]
//     #[allow(dead_code)]
//     pub fn new_48(prefix: P) -> Self {
//         let nt = NodeType::Node48(IndexNode::new());
//         Self { prefix, ntype: nt }
//     }
//
//     #[inline]
//     #[allow(dead_code)]
//     pub fn new_256(prefix: P) -> Self {
//         let nt = NodeType::Node256(DirectNode::new());
//         Self { prefix, ntype: nt }
//     }
//
//     pub fn value(&self) -> Option<&V> {
//         let NodeType::Leaf(value) = &self.ntype else {
//             return None;
//         };
//         Some(value)
//     }
//
//     #[allow(dead_code)]
//     pub fn value_mut(&mut self) -> Option<&mut V> {
//         let NodeType::Leaf(value) = &mut self.ntype else {
//             return None;
//         };
//         Some(value)
//     }
//
//     pub fn is_leaf(&self) -> bool {
//         matches!(&self.ntype, NodeType::Leaf(_))
//     }
//
//     pub fn is_inner(&self) -> bool {
//         !self.is_leaf()
//     }
//
//     pub fn num_children(&self) -> usize {
//         match &self.ntype {
//             NodeType::Node4(n) => n.num_children(),
//             NodeType::Node16(n) => n.num_children(),
//             NodeType::Node48(n) => n.num_children(),
//             NodeType::Node256(n) => n.num_children(),
//             NodeType::Leaf(_) => 0,
//         }
//     }
//     pub(crate) fn find_child(&self, key: u8) -> Option<&Node<P, V>> {
//         if self.num_children() == 0 {
//             return None;
//         }
//
//         match &self.ntype {
//             NodeType::Node4(km) => km.find_child(key),
//             NodeType::Node16(km) => km.find_child(key),
//             NodeType::Node48(km) => km.find_child(key),
//             NodeType::Node256(children) => children.find_child(key),
//             NodeType::Leaf(_) => None,
//         }
//     }
//
//     pub(crate) fn find_child_mut(&mut self, key: u8) -> Option<&mut Node<P, V>> {
//         match &mut self.ntype {
//             NodeType::Node4(km) => km.find_child_mut(key),
//             NodeType::Node16(km) => km.find_child_mut(key),
//             NodeType::Node48(km) => km.find_child_mut(key),
//             NodeType::Node256(children) => children.find_child_mut(key),
//             NodeType::Leaf(_) => None,
//         }
//     }
//
//     pub(crate) fn add_child(&mut self, key: u8, node: Node<P, V>) {
//         if self.is_full() {
//             self.grow();
//         }
//
//         match &mut self.ntype {
//             NodeType::Node4(km) => {
//                 km.add_child(key, node);
//             }
//             NodeType::Node16(km) => {
//                 km.add_child(key, node);
//             }
//             NodeType::Node48(im) => {
//                 im.add_child(key, node);
//             }
//             NodeType::Node256(pm) => {
//                 pm.add_child(key, node);
//             }
//             NodeType::Leaf(_) => unreachable!("Should not be possible."),
//         }
//     }
//
//     pub(crate) fn delete_child(&mut self, key: u8) -> Option<Node<P, V>> {
//         match &mut self.ntype {
//             NodeType::Node4(dm) => dm.delete_child(key),
//             NodeType::Node16(dm) => {
//                 let node = dm.delete_child(key);
//
//                 if self.num_children() < 5 {
//                     self.shrink();
//                 }
//                 node
//             }
//             NodeType::Node48(im) => {
//                 let node = im.delete_child(key);
//
//                 if self.num_children() < 17 {
//                     self.shrink();
//                 }
//
//                 // Return what we deleted.
//                 node
//             }
//             NodeType::Node256(pm) => {
//                 let node = pm.delete_child(key);
//                 if self.num_children() < 49 {
//                     self.shrink();
//                 }
//
//                 // Return what we deleted.
//                 node
//             }
//             NodeType::Leaf(_) => unreachable!("Should not be possible."),
//         }
//     }
//
//     #[inline]
//     fn is_full(&self) -> bool {
//         match &self.ntype {
//             NodeType::Node4(km) => self.num_children() >= km.width(),
//             NodeType::Node16(km) => self.num_children() >= km.width(),
//             NodeType::Node48(im) => self.num_children() >= im.width(),
//             // Should not be possible.
//             NodeType::Node256(_) => self.num_children() >= 256,
//             NodeType::Leaf(_) => unreachable!("Should not be possible."),
//         }
//     }
//
//     fn shrink(&mut self) {
//         match &mut self.ntype {
//             NodeType::Node4(_) => {
//                 unreachable!("Should never shrink a node4")
//             }
//             NodeType::Node16(km) => {
//                 self.ntype = NodeType::Node4(KeyedNode::from_resized_shrink(km));
//             }
//             NodeType::Node48(im) => {
//                 let new_node = NodeType::Node16(KeyedNode::from_indexed(im));
//                 self.ntype = new_node;
//             }
//             NodeType::Node256(dm) => {
//                 self.ntype = NodeType::Node48(IndexNode::from_direct(dm));
//             }
//             NodeType::Leaf(_) => unreachable!("Should not be possible."),
//         }
//     }
//
//     fn grow(&mut self) {
//         match &mut self.ntype {
//             NodeType::Node4(km) => {
//                 self.ntype = NodeType::Node16(KeyedNode::from_resized_grow(km))
//             }
//             NodeType::Node16(km) => self.ntype = NodeType::Node48(IndexNode::from_keyed(km)),
//             NodeType::Node48(im) => {
//                 self.ntype = NodeType::Node256(DirectNode::from_indexed(im));
//             }
//             NodeType::Node256 { .. } => {
//                 unreachable!("Should never grow a node256")
//             }
//             NodeType::Leaf(_) => unreachable!("Should not be possible."),
//         }
//     }
//
//     pub(crate) fn capacity(&self) -> usize {
//         match &self.ntype {
//             NodeType::Node4 { .. } => 4,
//             NodeType::Node16 { .. } => 16,
//             NodeType::Node48 { .. } => 48,
//             NodeType::Node256 { .. } => 256,
//             NodeType::Leaf(_) => 0,
//         }
//     }
//
//     #[allow(dead_code)]
//     pub(crate) fn free(&self) -> usize {
//         self.capacity() - self.num_children()
//     }
//
//     #[allow(dead_code)]
//     pub fn iter(&self) -> Box<dyn Iterator<Item=(u8, &Self)> + '_> {
//         return match &self.ntype {
//             NodeType::Node4(n) => Box::new(n.iter()),
//             NodeType::Node16(n) => Box::new(n.iter()),
//             NodeType::Node48(n) => Box::new(n.iter()),
//             NodeType::Node256(n) => Box::new(n.iter().map(|(k, v)| (k, v))),
//             NodeType::Leaf(_) => Box::new(std::iter::empty()),
//         };
//     }
// }
//
// impl BaseNode {
//     pub(crate) fn new(n_type: NodeType, prefix: &[u8]) -> Self {
//         let mut prefix_v: [u8; MAX_KEY_LEN] = [0; MAX_KEY_LEN];
//
//         assert!(prefix.len() <= MAX_KEY_LEN);
//         for (i, v) in prefix.iter().enumerate() {
//             prefix_v[i] = *v;
//         }
//
//         let meta = NodeMeta {
//             prefix_cnt: prefix.len() as u32,
//             count: 0,
//             prefix: prefix_v,
//             node_type: n_type,
//         };
//
//         BaseNode {
//             type_version_lock_obsolete: AtomicUsize::new(0),
//             meta,
//         }
//     }
//
//     pub(crate) fn get_type(&self) -> NodeType {
//         self.meta.node_type
//     }
//
//     // #[inline]
//     // pub(crate) fn read_lock(&self) -> Result<ReadGuard, ArtError> {
//     //
//     //     // 乐观锁实现
//     //     let version = self.type_version_lock_obsolete.load(Ordering::Acquire);
//     //
//     //     // #[cfg(test)]
//     //     // crate::utils::fail_point(ArtError::Locked(version))?;
//     //
//     //     if Self::is_locked(version) || Self::is_obsolete(version) {
//     //         return Err(ArtError::Locked(version));
//     //     }
//     //
//     //     Ok(ReadGuard::new(version, self))
//     // }
//
//     fn is_locked(version: usize) -> bool {
//         (version & 0b10) == 0b10
//     }
//
//     pub(crate) fn get_count(&self) -> usize {
//         self.meta.count as usize
//     }
//
//     fn is_obsolete(version: usize) -> bool {
//         (version & 1) == 1
//     }
//
//     pub(crate) fn prefix(&self) -> &[u8] {
//         self.meta.prefix[..self.meta.prefix_cnt as usize].as_ref()
//     }
//
//     pub(crate) fn insert_grow<CurT: NodeTrait<N, NUM_CHILDREN>, BiggerT: NodeTrait<N, NUM_CHILDREN>, >(
//         n: ConcreteReadGuard<CurT>,
//         parent: (u8, Option<ReadGuard>),
//         val: (u8, NodePtr),
//         allocator: &A,
//         guard: &Guard,
//     ) -> Result<(), ArtError> {
//         if !n.as_ref().is_full() {
//             if let Some(p) = parent.1 {
//                 p.unlock()?;
//             }
//
//             let mut write_n = n.upgrade().map_err(|v| v.1)?;
//
//             write_n.as_mut().insert(val.0, val.1);
//             return Ok(());
//         }
//
//         let p = parent
//             .1
//             .expect("parent node must present when current node is full");
//
//         let mut write_p = p.upgrade().map_err(|v| v.1)?;
//
//         let mut write_n = n.upgrade().map_err(|v| v.1)?;
//
//         let n_big = BaseNode::make_node::<BiggerT>(write_n.as_ref().base().prefix(), allocator)?;
//         write_n.as_ref().copy_to(unsafe { &mut *n_big });
//         unsafe { &mut *n_big }.insert(val.0, val.1);
//
//         write_p
//             .as_mut()
//             .change(parent.0, NodePtr::from_node(n_big as *mut BaseNode));
//
//         write_n.mark_obsolete();
//         let delete_n = write_n.as_mut() as *mut CurT as usize;
//         std::mem::forget(write_n);
//         let allocator: A = allocator.clone();
//         guard.defer(move || unsafe {
//             BaseNode::drop_node(delete_n as *mut BaseNode, allocator);
//         });
//         Ok(())
//     }
//
//     pub(crate) fn insert_and_unlock(
//         node: ReadGuard<'a>,
//         parent: (u8, Option<ReadGuard>),
//         val: (u8, NodePtr),
//         allocator: &'a A,
//         guard: &Guard,
//     ) -> Result<(), ArtError> {
//         match node.as_ref().get_type() {
//             NodeType::N4 => Self::insert_grow::<Node4, Node16, A>(
//                 node.into_concrete(),
//                 parent,
//                 val,
//                 allocator,
//                 guard,
//             ),
//             NodeType::N16 => Self::insert_grow::<Node16, Node48, A>(
//                 node.into_concrete(),
//                 parent,
//                 val,
//                 allocator,
//                 guard,
//             ),
//             NodeType::N48 => Self::insert_grow::<Node48, Node256, A>(
//                 node.into_concrete(),
//                 parent,
//                 val,
//                 allocator,
//                 guard,
//             ),
//             NodeType::N256 => Self::insert_grow::<Node256, Node256, A>(
//                 node.into_concrete(),
//                 parent,
//                 val,
//                 allocator,
//                 guard,
//             ),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use crate::router::tree::art::node::partials::ArrPartial;
    use super::*;

    // #[test]
    // fn test_n4() {
    //     let test_key: ArrPartial<16> = ArrPartial::key("abc".as_bytes());
    //
    //     let mut n4 = Node::new_4(test_key.clone());
    //     n4.add_child(5, Node::new_leaf(test_key.clone(), 1));
    //     n4.add_child(4, Node::new_leaf(test_key.clone(), 2));
    //     n4.add_child(3, Node::new_leaf(test_key.clone(), 3));
    //     n4.add_child(2, Node::new_leaf(test_key.clone(), 4));
    //
    //     assert_eq!(*n4.find_child(5).unwrap().value().unwrap(), 1);
    //     assert_eq!(*n4.find_child(4).unwrap().value().unwrap(), 2);
    //     assert_eq!(*n4.find_child(3).unwrap().value().unwrap(), 3);
    //     assert_eq!(*n4.find_child(2).unwrap().value().unwrap(), 4);
    //
    //     n4.delete_child(5);
    //     assert!(n4.find_child(5).is_none());
    //     assert_eq!(*n4.find_child(4).unwrap().value().unwrap(), 2);
    //     assert_eq!(*n4.find_child(3).unwrap().value().unwrap(), 3);
    //     assert_eq!(*n4.find_child(2).unwrap().value().unwrap(), 4);
    //
    //     n4.delete_child(2);
    //     assert!(n4.find_child(5).is_none());
    //     assert!(n4.find_child(2).is_none());
    //
    //     n4.add_child(2, Node::new_leaf(test_key, 4));
    //     n4.delete_child(3);
    //     assert!(n4.find_child(5).is_none());
    //     assert!(n4.find_child(3).is_none());
    // }
    //
    // #[test]
    // fn test_n16() {
    //     let test_key: ArrPartial<16> = ArrPartial::key("abc".as_bytes());
    //
    //     let mut n16 = Node::new_16(test_key.clone());
    //
    //     // Fill up the node with keys in reverse order.
    //     for i in (0..16).rev() {
    //         n16.add_child(i, Node::new_leaf(test_key.clone(), i));
    //     }
    //
    //     for i in 0..16 {
    //         assert_eq!(*n16.find_child(i).unwrap().value().unwrap(), i);
    //     }
    //
    //     // Delete from end doesn't affect position of others.
    //     n16.delete_child(15);
    //     n16.delete_child(14);
    //     assert!(n16.find_child(15).is_none());
    //     assert!(n16.find_child(14).is_none());
    //     for i in 0..14 {
    //         assert_eq!(*n16.find_child(i).unwrap().value().unwrap(), i);
    //     }
    //
    //     n16.delete_child(0);
    //     n16.delete_child(1);
    //     assert!(n16.find_child(0).is_none());
    //     assert!(n16.find_child(1).is_none());
    //     for i in 2..14 {
    //         assert_eq!(*n16.find_child(i).unwrap().value().unwrap(), i);
    //     }
    //
    //     // Delete from the middle
    //     n16.delete_child(5);
    //     n16.delete_child(6);
    //     assert!(n16.find_child(5).is_none());
    //     assert!(n16.find_child(6).is_none());
    //     for i in 2..5 {
    //         assert_eq!(*n16.find_child(i).unwrap().value().unwrap(), i);
    //     }
    //     for i in 7..14 {
    //         assert_eq!(*n16.find_child(i).unwrap().value().unwrap(), i);
    //     }
    // }
    //
    // #[test]
    // fn test_n48() {
    //     let test_key: ArrPartial<16> = ArrPartial::key("abc".as_bytes());
    //
    //     let mut n48 = Node::new_48(test_key.clone());
    //
    //     // indexes in n48 have no sort order, so we don't look at that
    //     for i in 0..48 {
    //         n48.add_child(i, Node::new_leaf(test_key.clone(), i));
    //     }
    //
    //     for i in 0..48 {
    //         assert_eq!(*n48.find_child(i).unwrap().value().unwrap(), i);
    //     }
    //
    //     n48.delete_child(47);
    //     n48.delete_child(46);
    //     assert!(n48.find_child(47).is_none());
    //     assert!(n48.find_child(46).is_none());
    //     for i in 0..46 {
    //         assert_eq!(*n48.find_child(i).unwrap().value().unwrap(), i);
    //     }
    // }
    //
    // #[test]
    // fn test_n_256() {
    //     let test_key: ArrPartial<16> = ArrPartial::key("abc".as_bytes());
    //
    //     let mut n256 = Node::new_256(test_key.clone());
    //
    //     for i in 0..=255 {
    //         n256.add_child(i, Node::new_leaf(test_key.clone(), i));
    //     }
    //     for i in 0..=255 {
    //         assert_eq!(*n256.find_child(i).unwrap().value().unwrap(), i);
    //     }
    //
    //     n256.delete_child(47);
    //     n256.delete_child(46);
    //     assert!(n256.find_child(47).is_none());
    //     assert!(n256.find_child(46).is_none());
    //     for i in 0..46 {
    //         assert_eq!(*n256.find_child(i).unwrap().value().unwrap(), i);
    //     }
    //     for i in 48..=255 {
    //         assert_eq!(*n256.find_child(i).unwrap().value().unwrap(), i);
    //     }
    // }
}
