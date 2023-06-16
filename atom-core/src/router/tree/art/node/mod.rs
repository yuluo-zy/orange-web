use std::sync::atomic::AtomicUsize;

const MAX_KEY_LEN: usize = 8;

type Prefix = [u8; MAX_KEY_LEN];

#[derive(Copy, Clone, Default, PartialOrd, PartialEq)]
pub(crate) enum NodeType {
    Node4 = 0,
    Node16 = 1,
    Node48 = 2,
    Node256 = 3,
}

pub(crate) struct NodeMeta {
    pub node_type: NodeType,
    pub node_prefix: Prefix, // todo 前缀
    pub count: u16,
    pub prefix_cnt: u16,
}

pub(crate) struct NodePtr {
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
    fn copy_to<N: Node>(&self, dst: &mut N);
    fn get_type() -> NodeType;
}

impl BaseNode {

    // pub fn make_node<N: NodeTrait>(prefix: &[u8], )
}


