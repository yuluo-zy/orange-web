use std::cell::UnsafeCell;
use std::cmp::min;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use crossbeam_epoch::Guard;
use crate::router::tree::art::node::keys::{Partial};
use crate::router::tree::art::node::{Node, NodeType, TreeNode};
use crate::router::tree::art::utils::{Backoff, TreeError};

pub trait PrefixTraits: Partial + PartialEq + Debug + for<'a> From<&'a [u8]> {}

impl<T: Partial + PartialEq + Debug + for<'a> From<&'a [u8]>> PrefixTraits for T {}

pub(crate) struct RawTree<P: PrefixTraits, V> {
    pub(crate) root: Node<P, V>,
}

unsafe impl<P: PrefixTraits, V> Send for RawTree<P, V> {}

unsafe impl<P: PrefixTraits, V> Sync for RawTree<P, V> {}

impl<P: PrefixTraits, V> Default for RawTree<P, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: PrefixTraits, V> RawTree<P, V> {
    pub fn new() -> Self {
        RawTree {
            root: Node::empty_leaf()
        }
    }
    #[inline]
    pub fn get(&self, key: &P, _guard: &Guard) -> Option<&V> {
        RawTree::get_inner(self.root.as_ref()?, key)
    }
    #[inline]
    fn get_inner<'a>(cur_node: &'a Node<P, V>, key: &P) -> Option<&'a V> {
        'outer: loop {
            let mut cur_node = if let Ok(node) =
                cur_node.read_lock() { node } else { continue; };

            let mut depth = 0;

            loop {
                let cur_ref = cur_node.as_ref();
                let prefix_common_match = cur_ref.prefix.prefix_length_key(key, depth);

                if prefix_common_match != cur_ref.prefix.len() {
                    return None;
                }

                if cur_ref.prefix.len() == key.length_at(depth) {
                    assert!(cur_ref.node_type() == NodeType::Leaf);
                    return cur_node.as_ref().value();
                }

                assert!(cur_ref.num_children() > 0);

                let k = key.at(depth + cur_ref.prefix.len());
                let sub_node = cur_ref.find_child(k)?;

                // 检查 查询子节点的时候， 当前节点是否被修改
                if cur_node.check_version().is_err() { continue 'outer; }
                depth += cur_ref.prefix.len();
                cur_node = if let Ok(node) = sub_node.read_lock() {
                    node
                } else {
                    continue 'outer;
                }
            }
        }
    }
    #[inline]
    pub fn get_mut(&mut self, key: &P, _guard: &Guard) -> Option<&mut V>

    {
        RawTree::get_inner_mut(self.root.as_mut()?, key)
    }

    #[inline]
    fn get_inner_mut<'a>(cur_node: &'a mut Node<P, V>, key: &P) -> Option<&'a mut V>
    {
        let mut cur_node = cur_node;
        let mut depth = 0;
        loop {
            let prefix_common_match = cur_node.prefix.prefix_length_key(key, depth);
            if prefix_common_match != cur_node.prefix.len() {
                return None;
            }

            if cur_node.prefix.len() == key.length_at(depth) {
                return cur_node.value_mut();
            }

            let k = key.at(depth + cur_node.prefix.len());
            depth += cur_node.prefix.len();
            cur_node = cur_node.find_child_mut(k)?;
        }
    }

    #[inline]
    pub fn insert(&self, key: &P, value: V, guard: &Guard) -> Option<V> {
        let backoff = Backoff::new();
        loop {
            match self.insert_inner(key, value, 0, guard) {
                Ok(v) => return v,
                Err(e) => match e {
                    TreeError::Locked | TreeError::VersionNotMatch => {
                        backoff.spin();
                        continue;
                    }
                    _ => { unreachable!() }
                },
            }
        }
    }
    #[inline]
    fn insert_inner(
        &self,
        key: &P,
        value: V,
        depth: usize,
        _guard: &Guard,
    ) -> Result<Option<V>, TreeError>
    {
        let node_lock = self.root.read_lock()?;

        if node_lock.as_ref().node_type() = NodeType::Empty { // 锁操作
            node_lock.as_mut().change(Node::new_leaf(key.partial_after(0), value));
            return Ok(None);
        }

        let mut node;

        loop {
            node = self.root.read_lock()?;

            let longest_common_prefix = node.as_ref().prefix.prefix_length_key(key, depth);

            let is_prefix_match =
                min(node.as_ref().prefix.len(), key.length_at(depth)) == longest_common_prefix;

            // 前缀匹配 并 当前节点与key完全匹配
            if is_prefix_match && node.as_ref().prefix.len() == key.length_at(depth) {
                if let TreeNode::Leaf(ref mut v) = &mut cur_node.tree_node {
                    return Some(std::mem::replace(v.value_mut()?, value));
                } else {
                    panic!("Node type mismatch")
                }
            }


        }

        // let longest_common_prefix = cur_node.prefix.prefix_length_key(key, depth);
        //
        // let is_prefix_match =
        //     min(cur_node.prefix.len(), key.length_at(depth)) == longest_common_prefix;
        //
        // // 前缀匹配 并 当前节点与key完全匹配
        // if is_prefix_match && cur_node.prefix.len() == key.length_at(depth) {
        //     if let TreeNode::Leaf(ref mut v) = &mut cur_node.tree_node {
        //         return Some(std::mem::replace(v.value_mut()?, value));
        //     } else {
        //         panic!("Node type mismatch")
        //     }
        // }
        //
        // // 分解节点
        // if !is_prefix_match {
        //     let n4 = Node::new(NodeType::Node4, cur_node.prefix.partial_before(longest_common_prefix));
        //
        //     let k1 = cur_node.prefix.at(longest_common_prefix);
        //     let k2 = key.at(depth + longest_common_prefix);
        //
        //     cur_node.prefix = cur_node.prefix.partial_after(longest_common_prefix);
        //     let replacement_current = std::mem::replace(cur_node, n4);
        //
        //     let new_leaf = Node::new_leaf(key.partial_after(depth + longest_common_prefix), value);
        //
        //     cur_node.add_child(k1, replacement_current);
        //     cur_node.add_child(k2, new_leaf);
        //
        //     return None;
        // }
        //
        // let k = key.at(depth + longest_common_prefix);
        //
        // let child_for_key = cur_node.find_child_mut(k);
        // if let Some(child) = child_for_key {
        //     return self.insert_inner(
        //         child,
        //         key,
        //         value,
        //         depth + longest_common_prefix,
        //     );
        // };
        //
        // assert!(cur_node.node_type() != NodeType::Leaf);
        // let new_leaf = Node::new_leaf(key.partial_after(depth + longest_common_prefix), value);
        // cur_node.add_child(k, new_leaf);
        // None
    }

    #[inline]
    pub fn remove(&mut self, key: &P) -> Option<V> {
        let Some(root) = self.root.as_mut() else {
            return None;
        };

        let prefix_common_match = root.prefix.prefix_length_key(key, 0);
        if prefix_common_match != root.prefix.len() {
            return None;
        }

        if root.node_type() == NodeType::Leaf {
            let mut stolen = self.root.take().expect("Node Leaf is null");
            let leaf = match stolen.tree_node {
                TreeNode::Leaf(node) => node.value,
                _ => unreachable!(),
            };
            return leaf;
        }

        let result = RawTree::remove_recurse(root, key, prefix_common_match);
        if root.node_type() != NodeType::Leaf && root.num_children() == 0 {
            self.root = None;
        }
        result
    }
    #[inline]
    fn remove_recurse(
        parent_node: &mut Node<P, V>,
        key: &P,
        depth: usize,
    ) -> Option<V> {
        // Seek the child that matches the key at this depth, which is the first character at the
        // depth we're at.
        let c = key.at(depth);
        let child_node = parent_node.find_child_mut(c)?;

        let prefix_common_match = child_node.prefix.prefix_length_key(key, depth);
        if prefix_common_match != child_node.prefix.len() {
            return None;
        }

        // If the child is a leaf, and the prefix matches the key, we can remove it from this parent
        // node. If the prefix does not match, then we have nothing to do here.
        if child_node.node_type() == NodeType::Leaf {
            if child_node.prefix.len() != (key.length_at(depth)) {
                return None;
            }
            let node = parent_node.delete_child(c).expect("child not found");
            let v = match node.tree_node {
                TreeNode::Leaf(v) => v.value,
                _ => unreachable!(),
            };
            return v;
        }

        // Otherwise, recurse down the branch in that direction.
        let result =
            RawTree::remove_recurse(child_node, key, depth + child_node.prefix.len());

        if result.is_some() && child_node.node_type() != NodeType::Leaf && child_node.num_children() == 0 {
            parent_node
                .delete_child(c)
                .expect("expected empty inner node to be deleted");
        }

        // TODO: 如果内部节点只有一个（叶子）子节点，则将其转换为叶子.

        result
    }
}

#[derive(Debug)]
pub struct NodeStats {
    width: usize,
    total_nodes: usize,
    total_children: usize,
    density: f64,
}

#[derive(Debug)]
pub struct TreeStats {
    pub node_stats: HashMap<usize, NodeStats>,
    pub num_leaves: usize,
    pub num_values: usize,
    pub num_inner_nodes: usize,
    pub total_density: f64,
    pub max_height: usize,
}

fn update_tree_stats<P: Partial + Clone, V>(tree_stats: &mut TreeStats, node: &Node<P, V>) {
    tree_stats
        .node_stats
        .entry(node.capacity())
        .and_modify(|e| {
            e.total_nodes += 1;
            e.total_children += node.num_children();
        })
        .or_insert(NodeStats {
            width: node.capacity(),
            total_nodes: 1,
            total_children: node.num_children(),
            density: 0.0,
        });
}
//
// #[cfg(test)]
// mod tests {
//     use std::collections::{btree_map, BTreeMap, BTreeSet};
//     use std::fmt::Debug;
//
//     use rand::seq::SliceRandom;
//     use rand::{thread_rng, Rng};
//     use crate::router::tree::art::node::keys::{Partial, RawKey};
//
//
//     use crate::router::tree::art::tree::RawTree;
//
//     #[test]
//     fn test_root_set_get() {
//         let mut q = RawTree::<RawKey<16>, i32>::new();
//         let key = RawKey::from_str("abc");
//         assert!(q.insert(&key, 1).is_none());
//         assert_eq!(q.get(&key), Some(&1));
//     }
//
//     #[test]
//     fn test_string_keys_get_set() {
//         let mut q = RawTree::<RawKey<16>, i32>::new();
//         q.insert(&RawKey::from_str("abcd"), 1);
//         q.insert(&RawKey::from_str("abc"), 2);
//         q.insert(&RawKey::from_str("abcde"), 3);
//         q.insert(&RawKey::from_str("xyz"), 4);
//         q.insert(&RawKey::from_str("xyz"), 5);
//         q.insert(&RawKey::from_str("axyz"), 6);
//         q.insert(&RawKey::from_str("1245zzz1245zzz1245zzz1245zzz"), 6);
//
//         assert_eq!(*q.get(&RawKey::from_str("abcd")).unwrap(), 1);
//         assert_eq!(*q.get(&RawKey::from_str("abc")).unwrap(), 2);
//         assert_eq!(*q.get(&RawKey::from_str("abcde")).unwrap(), 3);
//         assert_eq!(*q.get(&RawKey::from_str("1245zzz1245zzz1245zzz1245zzz")).unwrap(), 6);
//         assert_eq!(*q.get(&RawKey::from_str("xyz")).unwrap(), 5);
//
//         assert_eq!(q.remove(&RawKey::from_str("abcde")), Some(3));
//         assert_eq!(q.get(&RawKey::from_str("abcde")), None);
//         assert_eq!(*q.get(&RawKey::from_str("abc")).unwrap(), 2);
//         assert_eq!(*q.get(&RawKey::from_str("axyz")).unwrap(), 6);
//         assert_eq!(q.remove(&RawKey::from_str("abc")), Some(2));
//         assert_eq!(q.get(&RawKey::from_str("abc")), None);
//     }
//
//     #[test]
//     fn test_int_keys_get_set() {
//         let mut q = RawTree::<RawKey<16>, i32>::new();
//         q.insert(&500i32.into(), 3);
//         assert_eq!(q.get(&500i32.into()), Some(&3));
//         q.insert(&666i32.into(), 2);
//         assert_eq!(q.get(&666i32.into()), Some(&2));
//         q.insert(&1i32.into(), 1);
//         assert_eq!(q.get(&1i32.into()), Some(&1));
//     }
//
//     fn gen_random_string_keys<const S: usize>(
//         l1_prefix: usize,
//         l2_prefix: usize,
//         suffix: usize,
//     ) -> Vec<(RawKey<S>, String)> {
//         let mut keys = Vec::new();
//         let chars: Vec<char> = ('a'..='z').collect();
//         for i in 0..chars.len() {
//             let level1_prefix = chars[i].to_string().repeat(l1_prefix);
//             for i in 0..chars.len() {
//                 let level2_prefix = chars[i].to_string().repeat(l2_prefix);
//                 let key_prefix = level1_prefix.clone() + &level2_prefix;
//                 for _ in 0..=u8::MAX {
//                     let suffix: String = (0..suffix)
//                         .map(|_| chars[thread_rng().gen_range(0..chars.len())])
//                         .collect();
//                     let string = key_prefix.clone() + &suffix;
//                     let k = string.clone().into();
//                     keys.push((k, string));
//                 }
//             }
//         }
//
//         keys.shuffle(&mut thread_rng());
//         keys
//     }
//
//     #[test]
//     fn test_bulk_random_string_query() {
//         let mut tree = RawTree::<RawKey<16>, String>::new();
//         let keys = gen_random_string_keys(3, 2, 3);
//         let mut num_inserted = 0;
//         for (_i, key) in keys.iter().enumerate() {
//             // println!("{:?}", key.1.clone() );
//             if tree.insert(&key.0, key.1.clone()).is_none() {
//                 num_inserted += 1;
//                 assert!(tree.get(&key.0).is_some());
//             }
//         }
//         let mut rng = thread_rng();
//         for _i in 0..500_000_00 {
//             let entry = &keys[rng.gen_range(0..keys.len())];
//             let val = tree.get(&entry.0);
//             assert!(val.is_some());
//             assert_eq!(*val.unwrap(), entry.1);
//         }
//     }
//
//     #[test]
//     fn test_random_numeric_insert_get() {
//         let mut tree = RawTree::<RawKey<16>, u64>::new();
//         let count = 100_000;
//         let mut rng = thread_rng();
//         let mut keys_inserted = vec![];
//         for i in 0..count {
//             let value = i;
//             let rnd_key = rng.gen_range(0..count);
//             let rnd_key: RawKey<16> = rnd_key.into();
//             if tree.get(&rnd_key).is_none() && tree.insert(&rnd_key, value).is_none() {
//                 let result = tree.get(&rnd_key);
//                 assert!(result.is_some());
//                 assert_eq!(*result.unwrap(), value);
//                 keys_inserted.push((rnd_key, value));
//             }
//         }
//
//         // let stats = tree.get_tree_stats();
//         // assert_eq!(stats.num_values, keys_inserted.len());
//
//         for (key, value) in &keys_inserted {
//             let result = tree.get(key);
//             assert!(result.is_some());
//             assert_eq!(*result.unwrap(), *value, );
//         }
//     }
//
//     fn from_be_bytes_key(k: &Vec<u8>) -> u64 {
//         let k = if k.len() < 8 {
//             let mut new_k = vec![0; 8];
//             new_k[8 - k.len()..].copy_from_slice(k);
//             new_k
//         } else {
//             k.clone()
//         };
//         let k = k.as_slice();
//
//         u64::from_be_bytes(k[0..8].try_into().unwrap())
//     }
//
//     #[test]
//     // The following cases were found by fuzzing, and identified bugs in `remove`
//     fn test_delete_regressions() {
//         // DO_INSERT,12297829382473034287,72245244022401706
//         // DO_INSERT,12297829382473034410,5425513372477729450
//         // DO_DELETE,12297829382473056255,Some(5425513372477729450),None
//         let mut tree = RawTree::<RawKey<16>, usize>::new();
//         assert!(tree.insert(&RawKey::from(12297829382473034287usize), 72245244022401706usize).is_none());
//         assert!(tree.insert(&RawKey::from(12297829382473034410usize), 5425513372477729450usize).is_none());
//         // assert!(tree.remove(&ArrayKey::new_from_unsigned(12297829382473056255usize)).is_none());
//
//         let mut tree = RawTree::<RawKey<16>, usize>::new();
//         // DO_INSERT,0,8101975729639522304
//         // DO_INSERT,4934144,18374809624973934592
//         // DO_DELETE,0,None,Some(8101975729639522304)
//         assert!(tree.insert(&RawKey::from(0usize), 8101975729639522304usize).is_none());
//         assert!(tree.insert(&RawKey::from(4934144usize), 18374809624973934592usize).is_none());
//         assert_eq!(tree.get(&RawKey::from(0usize)), Some(&8101975729639522304usize));
//         assert_eq!(tree.remove(&RawKey::from(0usize)), Some(8101975729639522304usize));
//         assert_eq!(tree.get(&RawKey::from(4934144usize)), Some(&18374809624973934592usize));
//
//         // DO_INSERT,8102098874941833216,8101975729639522416
//         // DO_INSERT,8102099357864587376,18374810107896688752
//         // DO_DELETE,0,Some(8101975729639522416),None
//         let mut tree = RawTree::<RawKey<16>, usize>::new();
//         assert!(tree.insert(&RawKey::from(8102098874941833216usize), 8101975729639522416usize).is_none());
//         assert!(tree.insert(&RawKey::from(8102099357864587376usize), 18374810107896688752usize).is_none());
//         assert_eq!(tree.get(&RawKey::from(0usize)), None);
//         assert_eq!(tree.remove(&RawKey::from(0usize)), None);
//     }
//
//     #[test]
//     fn test_delete() {
//         // Insert a bunch of random keys and values into both a btree and our tree, then iterate
//         // over the btree and delete the keys from our tree. Then, iterate over our tree and make
//         // sure it's empty.
//         let mut tree = RawTree::<RawKey<16>, u64>::new();
//         let mut btree = BTreeMap::new();
//         let count = 5_000;
//         let mut rng = thread_rng();
//         for i in 0..count {
//             let _value = i;
//             let rnd_val = rng.gen_range(0..u64::MAX);
//             let rnd_key: RawKey<16> = rnd_val.into();
//             tree.insert(&rnd_key, rnd_val);
//             btree.insert(rnd_val, rnd_val);
//         }
//
//         for (key, value) in btree.iter() {
//             let key: RawKey<16> = (*key).into();
//             let get_result = tree.get(&key);
//             assert_eq!(
//                 get_result.cloned(),
//                 Some(*value),
//                 "Key with prefix {:?} not found in tree; it should be",
//                 key.partial_after(0).to_slice()
//             );
//             let result = tree.remove(&key);
//             assert_eq!(result, Some(*value));
//         }
//     }
// }