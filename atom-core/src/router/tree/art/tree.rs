use std::marker::PhantomData;
use std::sync::Arc;
use crate::router::tree::art::node::node::Node256;
use crate::router::tree::art::{ArtAllocator, TreeKeyTrait};
use crate::router::tree::art::node::BaseNode;

pub(crate) struct RawTree<K: TreeKeyTrait, V> {
    pub(crate) root: *const Node256,
    allocator: Arc<ArtAllocator>,
    _pt_key: PhantomData<K>,
    _pt_value: PhantomData<V>,
}

unsafe impl<K: TreeKeyTrait, V> Send for RawTree<K, V> {}

unsafe impl<K: TreeKeyTrait, V> Sync for RawTree<K, V> {}

impl<K: TreeKeyTrait, V> Default for RawTree<K, V> {
    fn default() -> Self {
        todo!()
    }
}

impl<K: TreeKeyTrait, V> Drop for RawTree<K, V> {
    fn drop(&mut self) {
        todo!()
    }
}

impl<K: TreeKeyTrait, V> RawTree<K, V> {
    pub fn new(allocator: Arc<ArtAllocator>) -> Self {
        RawTree {
            root: BaseNode::make_node::<Node256>(&[], allocator.clone())
                .expect("Can't allocate memory for root node!") as *const Node256,
            allocator,
            _pt_key: PhantomData,
            _pt_value: PhantomData,
        }
    }

    // #[inline]
    // fn insert_inner<F>(
    //     &self,
    //     k: &T,
    //     tid_func: &mut F,
    // ) -> Result<Option<usize>, ArtError>
    //     where
    //         F: FnMut(Option<usize>) -> usize,
    // {
    //     let mut parent_node = None;
    //     let mut next_node = self.root as *const BaseNode;
    //     let mut parent_key: u8;
    //     let mut node_key: u8 = 0;
    //     let mut level = 0;
    //
    //     let mut node;
    //
    //     loop {
    //         parent_key = node_key;
    //         node = unsafe { &*next_node }.read_lock()?;
    //
    //         let mut next_level = level;
    //         let res = self.check_prefix_not_match(node.as_ref(), k, &mut next_level);
    //         match res {
    //             None => {
    //                 level = next_level;
    //                 node_key = k.as_bytes()[level as usize];
    //
    //                 let next_node_tmp = node.as_ref().get_child(node_key);
    //
    //                 node.check_version()?;
    //
    //                 let next_node_tmp = if let Some(n) = next_node_tmp {
    //                     n
    //                 } else {
    //                     let new_leaf = {
    //                         if level == (MAX_KEY_LEN - 1) as u32 {
    //                             // last key, just insert the tid
    //                             NodePtr::from_tid(tid_func(None))
    //                         } else {
    //                             let new_prefix = k.as_bytes();
    //                             let n4 = BaseNode::make_node::<Node4>(
    //                                 &new_prefix[..k.len() - 1],
    //                                 &self.allocator,
    //                             )?;
    //                             unsafe { &mut *n4 }.insert(
    //                                 k.as_bytes()[k.len() - 1],
    //                                 NodePtr::from_tid(tid_func(None)),
    //                             );
    //                             NodePtr::from_node(n4 as *mut BaseNode)
    //                         }
    //                     };
    //
    //                     if let Err(e) = BaseNode::insert_and_unlock(
    //                         node,
    //                         (parent_key, parent_node),
    //                         (node_key, new_leaf),
    //                         &self.allocator,
    //                         guard,
    //                     ) {
    //                         if level != (MAX_KEY_LEN - 1) as u32 {
    //                             unsafe {
    //                                 BaseNode::drop_node(
    //                                     new_leaf.as_ptr() as *mut BaseNode,
    //                                     self.allocator.clone(),
    //                                 );
    //                             }
    //                         }
    //                         return Err(e);
    //                     }
    //
    //                     return Ok(None);
    //                 };
    //
    //                 if let Some(p) = parent_node {
    //                     p.unlock()?;
    //                 }
    //
    //                 if level == (MAX_KEY_LEN - 1) as u32 {
    //                     // At this point, the level must point to the last u8 of the key,
    //                     // meaning that we are updating an existing value.
    //
    //                     let old = node.as_ref().get_child(node_key).unwrap().as_tid();
    //                     let new = tid_func(Some(old));
    //                     if old == new {
    //                         node.check_version()?;
    //                         return Ok(Some(old));
    //                     }
    //
    //                     let mut write_n = node.upgrade().map_err(|(_n, v)| v)?;
    //
    //                     let old = write_n.as_mut().change(node_key, NodePtr::from_tid(new));
    //                     return Ok(Some(old.as_tid()));
    //                 }
    //                 next_node = next_node_tmp.as_ptr();
    //                 level += 1;
    //             }
    //
    //             Some(no_match_key) => {
    //                 let mut write_p = parent_node.unwrap().upgrade().map_err(|(_n, v)| v)?;
    //                 let mut write_n = node.upgrade().map_err(|(_n, v)| v)?;
    //
    //                 // 1) Create new node which will be parent of node, Set common prefix, level to this node
    //                 // let prefix_len = write_n.as_ref().prefix().len();
    //                 let new_middle_node = BaseNode::make_node::<Node4>(
    //                     write_n.as_ref().prefix()[0..next_level as usize].as_ref(),
    //                     &self.allocator,
    //                 )?;
    //
    //                 // 2)  add node and (tid, *k) as children
    //                 if next_level == (MAX_KEY_LEN - 1) as u32 {
    //                     // this is the last key, just insert to node
    //                     unsafe { &mut *new_middle_node }.insert(
    //                         k.as_bytes()[next_level as usize],
    //                         NodePtr::from_tid(tid_func(None)),
    //                     );
    //                 } else {
    //                     // otherwise create a new node
    //                     let single_new_node = BaseNode::make_node::<Node4>(
    //                         &k.as_bytes()[..k.len() - 1],
    //                         &self.allocator,
    //                     )?;
    //
    //                     unsafe { &mut *single_new_node }
    //                         .insert(k.as_bytes()[k.len() - 1], NodePtr::from_tid(tid_func(None)));
    //                     unsafe { &mut *new_middle_node }.insert(
    //                         k.as_bytes()[next_level as usize],
    //                         NodePtr::from_node(single_new_node as *const BaseNode),
    //                     );
    //                 }
    //
    //                 unsafe { &mut *new_middle_node }
    //                     .insert(no_match_key, NodePtr::from_node(write_n.as_mut()));
    //
    //                 // 3) update parentNode to point to the new node, unlock
    //                 write_p.as_mut().change(
    //                     parent_key,
    //                     NodePtr::from_node(new_middle_node as *mut BaseNode),
    //                 );
    //
    //                 return Ok(None);
    //             }
    //         }
    //         parent_node = Some(node);
    //     }
    // }
    //
    // #[inline]
    // pub(crate) fn insert(
    //     &self,
    //     k: T,
    //     tid: usize,
    //     // guard: &Guard,
    // ) -> Result<Option<usize>, OOMError> {
    //     let backoff = Backoff::new();
    //     loop {
    //         match self.insert_inner(&k, &mut |_| tid, guard) {
    //             Ok(v) => return Ok(v),
    //             Err(e) => match e {
    //                 ArtError::Locked(_) | ArtError::VersionNotMatch(_) => {
    //                     backoff.spin();
    //                     continue;
    //                 }
    //                 ArtError::Oom => return Err(OOMError::new()),
    //             },
    //         }
    //     }
    // }
}
