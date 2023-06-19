mod node;

use std::{alloc, ptr};
use std::alloc::{Allocator, Layout, LayoutError};
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use serde::de::StdError;
use crate::error;
use crate::error::Error;
use crate::router::tree::art::ArtAllocator;
use crate::router::tree::art::node::node::{NODE16TYPE, NODE256TYPE, NODE48TYPE, NODE4TYPE};

const MAX_KEY_LEN: usize = 8;

type Prefix = [u8; MAX_KEY_LEN];

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub enum NodeType {
    Node4 = 0,
    Node16 = 1,
    Node48 = 2,
    Node256 = 3,
}

pub struct NodeMeta {
    pub node_type: NodeType,
    // path compression时的前缀
    pub node_prefix: Prefix,
    // 前缀长度
    pub prefix_size: u16,
    pub count: u16,

}

impl From<LayoutError> for error::Error {
    fn from(value: LayoutError) -> Self {
      Error::new(value.to_string())
    }
}

impl NodeType {
    pub(crate) fn get_layout(&self) -> Result<Layout, LayoutError> {
        match *self {
            NodeType::Node4 => { NODE4TYPE }
            NodeType::Node16 => { NODE16TYPE }
            NodeType::Node48 => { NODE48TYPE }
            NodeType::Node256 => { NODE256TYPE }
        }
    }
}

pub struct NodePtr {
    pub node_id: usize,
    pub sub_node: *const BaseNode,
}

pub struct BaseNode {
    // 版本内容， 用来实现乐观锁
    pub type_version: AtomicUsize,
    pub meta: NodeMeta,
}

pub trait NodeTrait {
    fn base(&self) -> &BaseNode;
    fn base_mut(&mut self) -> &mut BaseNode;
    fn is_full(&self) -> bool;
    fn insert(&mut self, key: u8, node: NodePtr);
    fn change(&mut self, key: u8, val: NodePtr) -> NodePtr;
    fn get_child(&self, key: u8) -> Option<NodePtr>;
    fn remove(&mut self, k: u8);
    fn copy_to<N: NodeTrait>(&self, dst: &mut N);
    fn get_type() -> NodeType;
}

impl BaseNode {
    //
    pub fn make_node<N: NodeTrait>(prefix: &[u8], art_allocator: Arc<ArtAllocator>) -> Result<*mut N, Error> {
        let layout = N::get_type().get_layout()?;
        let ptr = art_allocator.allocate(layout).map_err(|e| Error::new(e.to_string()))?;
        let node_ptr = ptr.as_non_null_ptr().as_ptr() as *mut BaseNode;
        let node = BaseNode::new(N::get_type(), prefix);
        unsafe {
            ptr::write(node_ptr, node);
            Ok(node_ptr as *mut N)
        }
    }

    pub fn new(node_type: NodeType, prefix: &[u8]) -> Self {
        let mut prefix_temp: [u8; MAX_KEY_LEN] = [0; MAX_KEY_LEN];

        // 创建前缀
        for (index, value) in prefix.iter().enumerate() {
            prefix_temp[index] = *value;
            if index >= MAX_KEY_LEN - 1 {
                break;
            }
        }

        let meta = NodeMeta {
            node_type,
            node_prefix: prefix_temp,
            count: 0,
            prefix_size: prefix_temp.len() as u16,
        };

        BaseNode { type_version: AtomicUsize::new(0), meta }
    }

    pub fn get_type(&self) -> NodeType {
        return self.meta.node_type
    }

    pub fn get_count(&self) -> usize {
        return self.meta.count as usize;
    }

    pub fn prefix(&self) -> &[u8] {
        self.meta.node_prefix[..self.meta.prefix_size as usize].as_ref()
    }

    pub fn insert() {

    }

    pub fn insert_inner() {

    }



}


