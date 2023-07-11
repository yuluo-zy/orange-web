use std::cell::UnsafeCell;
use crate::router::tree::art::tree::{PrefixTraits, RawTree};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod tree;
pub mod node;
pub mod utils;
pub mod guard;

/// readme
/// https://zhuanlan.zhihu.com/p/142407392 实现相关文件
/// 并发算法 Multi-ART
/// https://zhuanlan.zhihu.com/p/65414186
/// 并发优化
/// https://oreki.blog.csdn.net/article/details/130119444?spm=1001.2101.3001.6650.1&utm_medium=distribute.pc_relevant.none-task-blog-2%7Edefault%7ECTRLIST%7ERate-1-130119444-blog-119805160.235%5Ev38%5Epc_relevant_sort_base1&depth_1-utm_source=distribute.pc_relevant.none-task-blog-2%7Edefault%7ECTRLIST%7ERate-1-130119444-blog-119805160.235%5Ev38%5Epc_relevant_sort_base1&utm_relevant_index=2

pub mod epoch {
    pub use crossbeam_epoch::{pin, Guard};
}

pub(crate) struct ArtTree<K: PrefixTraits,V: Copy> {
    inner: RawTree<K,V>,
}

impl<K: PrefixTraits, V: Copy> Default for ArtTree<K, V> {
    fn default() -> Self {
     Self {
         inner: RawTree::new()
     }
    }
}

impl<K: PrefixTraits, V: Copy> ArtTree<K,V> {

    pub fn pin(&self) -> epoch::Guard {
        crossbeam_epoch::pin()
    }

    // pub fn get<T: Into<K>>(&self, key: T, guard: &epoch::Guard) -> Option<&V> {
    //     let key = key.into();
    //     self.inner.get(&key, guard)
    // }

    // pub fn get_mut<T: Into<K>>(&mut self, key: &K, guard: &epoch::Guard) ->Option<&mut V> {
    //
    // }
    //
    pub fn insert<T: Into<K>>(&self, key: T, value: V, guard: &epoch::Guard)-> Option<V> {
        let key = key.into();
        self.inner.insert(&key, value, guard)
    }
}



#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::thread;
    use rand::prelude::StdRng;
    use rand::seq::SliceRandom;
    use rand::{thread_rng, Rng, SeedableRng};
    use crate::router::tree::art::node::keys::RawKey;
    use crate::router::tree::art::tree::RawTree;

    #[test]
    fn test_concurrent_insert() {
        let key_cnt_per_thread = 5;
        let n_thread = 3;
        let mut key_space = Vec::with_capacity(key_cnt_per_thread * n_thread);
        for i in 0..key_space.capacity() {
            key_space.push(i);
        }
        let mut r = StdRng::seed_from_u64(42);
        key_space.shuffle(&mut r);

        let key_space = Arc::new(key_space);

        let tree = Arc::new(RawTree::<RawKey<8>, usize>::new());

        let mut handlers = Vec::new();
        for t in 0..n_thread {
            let key_space = key_space.clone();
            let mut tree = tree.clone();

            handlers.push(thread::spawn(move || {
                let guard = crossbeam_epoch::pin();
                for i in 0..key_cnt_per_thread {
                    let idx = t * key_cnt_per_thread + i;
                    let val = key_space[idx];
                    tree.insert(&RawKey::from(val), val, &guard);
                }
            }));
        }

        for h in handlers.into_iter() {
            h.join().unwrap();
        }

        // let guard = crossbeam_epoch::pin();
        // for v in key_space.iter() {
        //     let val = tree.get(&RawKey::from(*v), &guard).unwrap();
        //     assert_eq!(*val, *v);
        // }
    }

    // #[test]
    // fn test_concurrent_insert_read() {
    //     let key_cnt_per_thread = 5_000;
    //     let w_thread = 2;
    //     let mut key_space = Vec::with_capacity(key_cnt_per_thread * w_thread);
    //     for i in 0..key_space.capacity() {
    //         key_space.push(i);
    //     }
    //
    //     let mut r = StdRng::seed_from_u64(42);
    //     key_space.shuffle(&mut r);
    //
    //     let key_space = Arc::new(key_space);
    //
    //     let tree = Arc::new(RawTree::default());
    //
    //     let mut handlers = Vec::new();
    //     for t in 0..w_thread {
    //         let key_space = key_space.clone();
    //         let tree = tree.clone();
    //         handlers.push(thread::spawn(move || {
    //             let guard = crossbeam_epoch::pin();
    //             for i in 0..key_cnt_per_thread {
    //                 let idx = t * key_cnt_per_thread + i;
    //                 let val = key_space[idx];
    //                 tree.insert(GeneralKey::key_from(val), val, &guard).unwrap();
    //             }
    //         }));
    //     }
    //
    //     let r_thread = 2;
    //     for t in 0..r_thread {
    //         let tree = tree.clone();
    //         handlers.push(thread::spawn(move || {
    //             let mut r = StdRng::seed_from_u64(10 + t);
    //             let guard = crossbeam_epoch::pin();
    //             for _i in 0..key_cnt_per_thread {
    //                 let val = r.gen_range(0..(key_cnt_per_thread * w_thread));
    //                 if let Some(v) = tree.get(&GeneralKey::key_from(val), &guard) {
    //                     assert_eq!(v, val);
    //                 }
    //             }
    //         }));
    //     }
    //
    //     for h in handlers.into_iter() {
    //         h.join().unwrap();
    //     }
    //
    //     let guard = crossbeam_epoch::pin();
    //     for v in key_space.iter() {
    //         let val = tree.get(&GeneralKey::key_from(*v), &guard).unwrap();
    //         assert_eq!(val, *v);
    //     }
    // }
}