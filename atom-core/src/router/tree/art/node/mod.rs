use std::sync::atomic::{AtomicUsize, Ordering};
use crate::router::tree::art::node::bit_set::{Bitset16, Bitset64, Bitset8};
use crate::router::tree::art::node::direct_node::DirectNode;
use crate::router::tree::art::node::index_node::IndexNode;
use crate::router::tree::art::node::node::KeyedNode;
use std::marker::PhantomData;
use crossbeam_epoch::Guard;
use rand::distributions::Open01;
use crate::router::tree::art::guard::{ReadGuard};
use crate::router::tree::art::node::keys::Partial;
use crate::router::tree::art::node::leaf_node::NodeLeaf;
use crate::router::tree::art::utils::TreeError;

pub mod keys;
pub mod bit_set;
pub mod bit_array;
pub mod node;
pub mod direct_node;
pub mod index_node;
pub mod leaf_node;

pub(crate) const MAX_KEY_LEN: usize = 8;

pub(crate) type Prefix = [u8; MAX_KEY_LEN];

const NODE_TYPE_NONE: usize = 0;
const NODE_TYPE_N4: usize = 1;
const NODE_TYPE_N16: usize = 2;
const NODE_TYPE_N48: usize = 3;
const NODE_TYPE_N256: usize = 4;
const NODE_TYPE_LEAF: usize = 5;
const NODE_TYPE_MASK: usize = 7;
const NODE_PTR_MASK: usize = usize::MAX - NODE_TYPE_MASK;

pub trait NodeTrait<N> {
    fn add_child(&mut self, key: u8, node: N);
    fn update_child(&mut self, key: u8, node: N);
    fn find_child(&self, key: u8) -> Option<&N>;
    fn find_child_mut(&mut self, key: u8) -> Option<&mut N>;
    fn delete_child(&mut self, key: u8) -> Option<N>;
    fn num_children(&self) -> usize;
    fn width(&self) -> usize;
}

pub(crate) enum TreeNode<P: Partial, V> {
    Empty,
    Leaf(NodeLeaf<P, V>),
    Node4(KeyedNode<Node<P, V>, 4, Bitset8<1>>),
    Node16(KeyedNode<Node<P, V>, 16, Bitset16<1>>),
    Node48(IndexNode<Node<P, V>, 48, Bitset64<1>>),
    Node256(DirectNode<Node<P, V>>),
}

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum NodeType {
    Empty,
    Leaf,
    Node4,
    Node16,
    Node48,
    Node256,
}

pub(crate) struct Node<P: Partial, V> {
    pub(crate) prefix: P,
    // 2b type | 60b version | 1b lock | 1b obsolete
    pub(crate) type_version_lock_obsolete: AtomicUsize,
    pub(crate) tree_node: TreeNode<P, V>,
    value: PhantomData<V>,
}


impl<P: Partial, V> Node<P, V> {
    pub(crate) fn new(node_type: NodeType, prefix: P) -> Self {
        let node = match node_type {
            NodeType::Node4 => { TreeNode::Node4(KeyedNode::new()) }
            NodeType::Node16 => { TreeNode::Node16(KeyedNode::new()) }
            NodeType::Node48 => { TreeNode::Node48(IndexNode::new()) }
            NodeType::Node256 => { TreeNode::Node256(DirectNode::new()) }
            _ => { unreachable!() }
        };

        Self {
            prefix,
            type_version_lock_obsolete: AtomicUsize::new(0),
            tree_node: node,
            value: PhantomData,
        }
    }

    pub(crate) fn new_leaf(prefix: P, value: V) -> Self {
        let node = TreeNode::Leaf(NodeLeaf { kay: prefix.partial_after(0), value: Some(value) });

        Self {
            prefix,
            type_version_lock_obsolete: AtomicUsize::new(0),
            tree_node: node,
            value: PhantomData,
        }
    }

    pub(crate) fn empty_leaf() -> Self {
        let node = TreeNode::Empty;

        Self {
            prefix: P::empty(),
            type_version_lock_obsolete: AtomicUsize::new(0),
            tree_node: node,
            value: PhantomData,
        }
    }


    #[inline]
    pub(crate) fn read_lock(&self) -> Result<ReadGuard<P, V>, TreeError> {

        // 乐观锁实现
        let version = self.type_version_lock_obsolete.load(Ordering::Acquire);
        if Self::is_locked(version) || Self::is_obsolete(version) {
            return Err(TreeError::Locked);
        }

        Ok(ReadGuard::new(version, self))
    }

    fn is_locked(version: usize) -> bool {
        (version & 0b10) == 0b10
    }

    pub fn set_version(&mut self, version: AtomicUsize) {
        self.type_version_lock_obsolete = version;
    }

    fn is_obsolete(version: usize) -> bool {
        (version & 1) == 1
    }

    pub(crate) fn prefix(&self) -> &[u8] {
        self.prefix.to_slice()
    }

    pub fn value(&self) -> Option<&V> {
        let TreeNode::Leaf(value) = &self.tree_node else {
            return None;
        };
        value.value_ref()
    }

    #[allow(dead_code)]
    pub fn value_mut(&mut self) -> Option<&mut V> {
        let TreeNode::Leaf(value) = &mut self.tree_node else {
            return None;
        };
        value.value_mut()
    }

    pub fn find_child(&self, key: u8) -> Option<&Node<P, V>> {
        if self.num_children() == 0 {
            return None;
        }

        match &self.tree_node {
            TreeNode::Node4(km) => km.find_child(key),
            TreeNode::Node16(km) => km.find_child(key),
            TreeNode::Node48(km) => km.find_child(key),
            TreeNode::Node256(children) => children.find_child(key),
            _ => None,
        }
    }
    pub fn find_child_mut(&mut self, key: u8) -> Option<&mut Node<P, V>> {
        match &mut self.tree_node {
            TreeNode::Node4(n4) => { n4.find_child_mut(key) }
            TreeNode::Node16(n16) => { n16.find_child_mut(key) }
            TreeNode::Node48(n48) => { n48.find_child_mut(key) }
            TreeNode::Node256(n256) => { n256.find_child_mut(key) }
            _ => { None }
        }
    }

    pub fn delete_child(&mut self, key: u8) -> Option<Node<P, V>> {
        match &mut self.tree_node {
            TreeNode::Node4(n4) => {
                n4.delete_child(key)
            }
            TreeNode::Node16(n16) => {
                let node = n16.delete_child(key);
                if self.num_children() < 5 {
                    self.shrink();
                }
                node
            }
            TreeNode::Node48(n48) => {
                let node = n48.delete_child(key);
                if self.num_children() < 17 {
                    self.shrink();
                }

                // Return what we deleted.
                node
            }
            TreeNode::Node256(n256) => {
                let node = n256.delete_child(key);
                if self.num_children() < 49 {
                    self.shrink();
                }
                node
            }
            _ => { None }
        }
    }


    pub(crate) fn add_child(&mut self, key: u8, node: Node<P, V>) {
        // if self.is_full() {
        //     self.grow();
        // }

        match &mut self.tree_node {
            TreeNode::Node4(km) => {
                km.add_child(key, node);
            }
            TreeNode::Node16(km) => {
                km.add_child(key, node);
            }
            TreeNode::Node48(im) => {
                im.add_child(key, node);
            }
            TreeNode::Node256(pm) => {
                pm.add_child(key, node);
            }
            _ => unreachable!("Should not be possible."),
        }
    }


    #[inline]
    fn is_full(&self) -> bool {
        match &self.tree_node {
            TreeNode::Node4(km) => self.num_children() >= km.width(),
            TreeNode::Node16(km) => self.num_children() >= km.width(),
            TreeNode::Node48(im) => self.num_children() >= im.width(),
            // Should not be possible.
            TreeNode::Node256(_) => self.num_children() >= 256,
            _ => unreachable!("Should not be possible."),
        }
    }

    fn shrink(&mut self) {
        match &mut self.tree_node {
            TreeNode::Node4(_) => {
                unreachable!("Should never shrink a node4")
            }
            TreeNode::Node16(km) => {
                self.tree_node = TreeNode::Node4(KeyedNode::from_resized_shrink(km));
            }
            TreeNode::Node48(im) => {
                let new_node = TreeNode::Node16(KeyedNode::from_indexed(im));
                self.tree_node = new_node;
            }
            TreeNode::Node256(dm) => {
                self.tree_node = TreeNode::Node48(IndexNode::from_direct(dm));
            }
            _ => unreachable!("Should not be possible."),
        }
    }

    fn grow(node: &mut Node<P, V>, guard: &Guard) {
        match &mut node.tree_node {
            TreeNode::Node4(km) => {
                node.tree_node = TreeNode::Node16(KeyedNode::from_resized_grow(km));
                km.children.clear();
                // guard.defer( || { km.children.clear(); })
            }
            TreeNode::Node16(km) => {
                node.tree_node = TreeNode::Node48(IndexNode::from_keyed(km));
                km.children.clear();
                // guard.defer( || { km.children.clear(); })
            }
            TreeNode::Node48(im) => {
                node.tree_node = TreeNode::Node256(DirectNode::from_indexed(im));
            }
            TreeNode::Node256 { .. } => {
                unreachable!("Should never grow a node256")
            }
            _ => unreachable!("Should not be possible."),
        }
    }

    pub(crate) fn capacity(&self) -> usize {
        match &self.tree_node {
            TreeNode::Node4 { .. } => 4,
            TreeNode::Node16 { .. } => 16,
            TreeNode::Node48 { .. } => 48,
            TreeNode::Node256 { .. } => 256,
            _ => 0,
        }
    }
    pub(crate) fn node_type(&self) -> NodeType {
        match &self.tree_node {
            TreeNode::Node4 { .. } => NodeType::Node4,
            TreeNode::Node16 { .. } => NodeType::Node16,
            TreeNode::Node48 { .. } => NodeType::Node48,
            TreeNode::Node256 { .. } => NodeType::Node256,
            TreeNode::Leaf(_) => NodeType::Leaf,
            TreeNode::Empty => NodeType::Empty,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn free(&self) -> usize {
        self.capacity() - self.num_children()
    }

    pub fn num_children(&self) -> usize {
        match &self.tree_node {
            TreeNode::Node4(n) => n.num_children(),
            TreeNode::Node16(n) => n.num_children(),
            TreeNode::Node48(n) => n.num_children(),
            TreeNode::Node256(n) => n.num_children(),
            _ => 0,
        }
    }

    pub fn change(&mut self, node: Node<P, V>) {
        self.tree_node = node.tree_node;
        self.prefix = node.prefix;
    }

    // #[allow(dead_code)]
    // pub fn iter(&self) -> Box<dyn Iterator<Item=(u8, &Self)> + '_> {
    //     return match &self.ntype {
    //         NodeType::Node4(n) => Box::new(n.iter()),
    //         NodeType::Node16(n) => Box::new(n.iter()),
    //         NodeType::Node48(n) => Box::new(n.iter()),
    //         NodeType::Node256(n) => Box::new(n.iter().map(|(k, v)| (k, v))),
    //         NodeType::Leaf(_) => Box::new(std::iter::empty()),
    //     };
    // }

    pub(crate) fn insert_unlock<'a>(node: &ReadGuard<'a, P, V>, val: (u8, Node<P, V>), _guard: &Guard) -> Result<(), TreeError> {
        // 解决 尚未充满的情况
        if !node.as_ref().is_full() {

            let mut write_node = node.upgrade().map_err(|v| v.1)?;
            write_node.as_mut().add_child(val.0, val.1);
            return Ok(());
        };

        // 针对节点充满的情况需要进行添加操作
        // let p_guard = parent.1.expect("parent node cannot find");
        // 节点充满的状态下, 则父节点需要存在, 用来锁定, 防止并发读写的时候出现问题
        // p_guard.upgrade().map_err(|v| v.1)?;
        let mut write_n = node.upgrade().map_err(|v| v.1)?;
        Self::grow(write_n.as_mut(), _guard);
        write_n.as_mut().add_child(val.0, val.1);
        Ok(())
    }

    pub(crate) fn update_unlock(
        node: &ReadGuard<P, V>,
        val: (u8, Node<P, V>),
        _guard: &Guard,
    ) -> Result<(), TreeError> {
        // 判断是否是 空节点的问题 && 判断 是否是 叶子节点 来完成 原地替换
        if node.as_ref().node_type() == NodeType::Empty
            ||( node.as_ref().node_type() == NodeType::Leaf && val.1.node_type() == NodeType::Leaf){

            let mut write_node = node.upgrade().map_err(|v| v.1)?;
            write_node.as_mut().change(val.1);
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::router::tree::art::node::keys::RawKey;
    use crate::router::tree::art::node::NodeType::{Node16, Node4};
    use super::*;

    #[test]
    fn test_n4() {
        let test_key: RawKey<16> = RawKey::from_str("abc");

        let mut n4 = Node::new(NodeType::Node4, test_key.clone());
        n4.add_child(5, Node::new_leaf(test_key.clone(), 1));
        n4.add_child(4, Node::new_leaf(test_key.clone(), 2));
        n4.add_child(3, Node::new_leaf(test_key.clone(), 3));
        n4.add_child(2, Node::new_leaf(test_key.clone(), 4));

        assert_eq!(*n4.find_child(5).unwrap().value().unwrap(), 1);
        assert_eq!(*n4.find_child(4).unwrap().value().unwrap(), 2);
        assert_eq!(*n4.find_child(3).unwrap().value().unwrap(), 3);
        assert_eq!(*n4.find_child(2).unwrap().value().unwrap(), 4);

        n4.delete_child(5);
        assert!(n4.find_child(5).is_none());
        assert_eq!(*n4.find_child(4).unwrap().value().unwrap(), 2);
        assert_eq!(*n4.find_child(3).unwrap().value().unwrap(), 3);
        assert_eq!(*n4.find_child(2).unwrap().value().unwrap(), 4);

        n4.delete_child(2);
        assert!(n4.find_child(5).is_none());
        assert!(n4.find_child(2).is_none());

        n4.add_child(2, Node::new_leaf(test_key.clone(), 4));
        n4.delete_child(3);
        assert!(n4.find_child(5).is_none());
        assert!(n4.find_child(3).is_none());
    }

    #[test]
    fn test_n16() {
        let test_key: RawKey<16> = RawKey::from_str("abc");

        let mut n16 = Node::new(NodeType::Node16, test_key.clone());

        // Fill up the node with keys in reverse order.
        for i in (0..16).rev() {
            n16.add_child(i, Node::new_leaf(test_key.clone(), i));
        }

        for i in 0..16 {
            assert_eq!(*n16.find_child(i).unwrap().value().unwrap(), i);
        }

        // Delete from end doesn't affect position of others.
        n16.delete_child(15);
        n16.delete_child(14);
        assert!(n16.find_child(15).is_none());
        assert!(n16.find_child(14).is_none());
        for i in 0..14 {
            assert_eq!(*n16.find_child(i).unwrap().value().unwrap(), i);
        }

        n16.delete_child(0);
        n16.delete_child(1);
        assert!(n16.find_child(0).is_none());
        assert!(n16.find_child(1).is_none());
        for i in 2..14 {
            assert_eq!(*n16.find_child(i).unwrap().value().unwrap(), i);
        }

        // Delete from the middle
        n16.delete_child(5);
        n16.delete_child(6);
        assert!(n16.find_child(5).is_none());
        assert!(n16.find_child(6).is_none());
        for i in 2..5 {
            assert_eq!(*n16.find_child(i).unwrap().value().unwrap(), i);
        }
        for i in 7..14 {
            assert_eq!(*n16.find_child(i).unwrap().value().unwrap(), i);
        }
    }

    #[test]
    fn test_n48() {
        let test_key: RawKey<16> = RawKey::from_str("abc");

        let mut n48 = Node::new(NodeType::Node48, test_key.clone());

        // indexes in n48 have no sort order, so we don't look at that
        for i in 0..48 {
            n48.add_child(i, Node::new_leaf(test_key.clone(), i));
        }

        for i in 0..48 {
            assert_eq!(*n48.find_child(i).unwrap().value().unwrap(), i);
        }

        n48.delete_child(47);
        n48.delete_child(46);
        assert!(n48.find_child(47).is_none());
        assert!(n48.find_child(46).is_none());
        for i in 0..46 {
            assert_eq!(*n48.find_child(i).unwrap().value().unwrap(), i);
        }
    }

    #[test]
    fn test_n_256() {
        let test_key: RawKey<16> = RawKey::from_str("abc");

        let mut n256 = Node::new(NodeType::Node256, test_key.clone());

        for i in 0..=255 {
            n256.add_child(i, Node::new_leaf(test_key.clone(), i));
        }
        for i in 0..=255 {
            assert_eq!(*n256.find_child(i).unwrap().value().unwrap(), i);
        }

        n256.delete_child(47);
        n256.delete_child(46);
        assert!(n256.find_child(47).is_none());
        assert!(n256.find_child(46).is_none());
        for i in 0..46 {
            assert_eq!(*n256.find_child(i).unwrap().value().unwrap(), i);
        }
        for i in 48..=255 {
            assert_eq!(*n256.find_child(i).unwrap().value().unwrap(), i);
        }
    }
}
