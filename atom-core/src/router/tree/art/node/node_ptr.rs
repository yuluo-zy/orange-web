use crate::router::tree::art::node::BaseNode;
use crate::router::tree::art::TreeKeyTrait;

#[derive(Clone, Copy)]
pub(crate) union NodePtr {
    ptr: *const BaseNode,
    sub_node: *const BaseNode,
}

impl NodePtr {
    #[inline]
    pub(crate) fn from_sub(ptr: *const BaseNode) -> Self {
        Self { sub_node: ptr }
    }

    #[inline]
    pub(crate) fn from_node<K: TreeKeyTrait, V>(ptr: *const BaseNode) -> Self {
        Self {
            ptr
        }
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const BaseNode {
        unsafe { self.ptr }
    }

    #[inline]
    pub(crate) fn as_sub_ptr(&self) -> *const BaseNode {
        unsafe { self.sub_node }
    }
}
