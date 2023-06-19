use std::alloc::{Layout, LayoutError};
use std::mem;
use crate::router::tree::art::node::{BaseNode, NodePtr, NodeTrait, NodeType};

pub(crate) const NODE4TYPE: Result<Layout, LayoutError> = Layout::from_size_align(mem::size_of::<Node4>(),
                                                                                  mem::align_of::<Node4>(), );
pub(crate) const NODE16TYPE: Result<Layout, LayoutError> = Layout::from_size_align(mem::size_of::<Node16>(),
                                                                                   mem::align_of::<Node16>(), );
pub(crate) const NODE48TYPE: Result<Layout, LayoutError> = Layout::from_size_align(mem::size_of::<Node48>(),
                                                                                   mem::align_of::<Node48>(), );
pub(crate) const NODE256TYPE: Result<Layout, LayoutError> = Layout::from_size_align(mem::size_of::<Node256>(),
                                                                                    mem::align_of::<Node256>(), );

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

    fn get_child(&self, key: u8) -> Option<NodePtr> {
        for i in 0..self.base.meta.count {
            if self.keys[i as usize] == key {
                let child = self.children[i as usize];
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

    fn copy_to<N: NodeTrait>(&self, dst: &mut N) {
        for i in 0..self.base.meta.count {
            dst.insert(self.keys[i as usize], self.children[i as usize]);
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
pub(crate) struct Node16 {
    base: BaseNode,
    children: [NodePtr; 16],
    keys: [u8; 16],
}

#[repr(C)]
#[repr(align(8))]
pub(crate) struct Node48 {
    base: BaseNode,
    pub(crate) child_idx: [u8; 256],
    next_empty: u8,
    children: [NodePtr; 48],
}
#[repr(C)]
#[repr(align(8))]
pub(crate) struct Node256 {
    base: BaseNode,
    key_mask: [u8; 32],
    children: [NodePtr; 256],
}
