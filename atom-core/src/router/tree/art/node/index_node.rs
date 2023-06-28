use std::mem::MaybeUninit;
use crate::router::tree::art::node::bit_array::BitArray;
use crate::router::tree::art::node::bit_set::{Bitset64, BitsetTrait};
use crate::router::tree::art::node::direct_node::DirectNode;
use crate::router::tree::art::node::node::KeyedNode;
use crate::router::tree::art::node::NodeTrait;

/// A mapping from keys to separate child pointers. 256 keys, usually 48 children.
pub struct IndexNode <N, const WIDTH: usize, Bitset: BitsetTrait> {
    pub(crate) child_ptr_indexes: BitArray<u8, 256, Bitset64<4>>,
    pub(crate) children: BitArray<N, WIDTH, Bitset>,
    pub(crate) num_children: u8,
}

impl<N, const WIDTH: usize, Bitset: BitsetTrait> Default for IndexNode<N, WIDTH, Bitset> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N, const WIDTH: usize, Bitset: BitsetTrait> IndexNode<N, WIDTH, Bitset> {
    pub fn new() -> Self {
        Self {
            child_ptr_indexes: Default::default(),
            children: BitArray::new(),
            num_children: 0,
        }
    }

    pub(crate) fn from_direct(dm: &mut DirectNode<N>) -> Self {
        let mut indexed = IndexNode::new();

        let keys: Vec<usize> = dm.children.iter_keys().collect();
        for key in keys {
            let child = dm.children.erase(key).unwrap();
            indexed.add_child(key as u8, child);
        }
        indexed
    }

    pub fn from_keyed<const KM_WIDTH: usize, FromBitset: BitsetTrait>(
        km: &mut KeyedNode<N, KM_WIDTH, FromBitset>,
    ) -> Self {
        let mut im: IndexNode<N, WIDTH, Bitset> = IndexNode::new();
        for i in 0..KM_WIDTH {
            let Some(stolen) = km.children.erase(i) else {
                continue;
            };
            im.add_child(km.keys[i], stolen);
        }
        km.children.clear();
        km.num_children = 0;
        im
    }

    pub(crate) fn move_into<const NEW_WIDTH: usize, NM: NodeTrait<N>>(
        &mut self,
        nm: &mut NM,
    ) {
        for (key, pos) in self.child_ptr_indexes.iter() {
            let node = self.children.erase(*pos as usize).unwrap();
            nm.add_child(key as u8, node);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (u8, &N)> {
        self.child_ptr_indexes
            .iter()
            .map(move |(key, pos)| (key as u8, &self.children[*pos as usize]))
    }
}

impl<N, const WIDTH: usize, Bitset: BitsetTrait> NodeTrait<N>
for IndexNode<N, WIDTH, Bitset>
{
    fn add_child(&mut self, key: u8, node: N) {
        let pos = self
            .children
            .first_empty()
            .expect("No empty slots in IndexedMapping; full");
        self.child_ptr_indexes.set(key as usize, pos as u8);
        self.children.set(pos, node);
        self.num_children += 1;
    }

    fn update_child(&mut self, key: u8, node: N) {
        if let Some(pos) = self.child_ptr_indexes.get(key as usize) {
            self.children.set(*pos as usize, node);
        }
    }

    fn find_child(&self, key: u8) -> Option<&N> {
        if let Some(pos) = self.child_ptr_indexes.get(key as usize) {
            return self.children.get(*pos as usize);
        }
        None
    }

    fn find_child_mut(&mut self, key: u8) -> Option<&mut N> {
        if let Some(pos) = self.child_ptr_indexes.get(key as usize) {
            return self.children.get_mut(*pos as usize);
        }
        None
    }

    fn delete_child(&mut self, key: u8) -> Option<N> {
        let pos = self.child_ptr_indexes.erase(key as usize)?;

        let old = self.children.erase(pos as usize);
        self.num_children -= 1;

        // Return what we deleted.
        old
    }

    fn num_children(&self) -> usize {
        self.num_children as usize
    }

    #[inline]
    fn width(&self) -> u16 {
        WIDTH as u16
    }
}

impl<N, const WIDTH: usize, Bitset: BitsetTrait> Drop for IndexNode<N, WIDTH, Bitset> {
    fn drop(&mut self) {
        if self.num_children == 0 {
            return;
        }
        self.num_children = 0;
        self.child_ptr_indexes.clear();
        self.children.clear();
    }
}

#[cfg(test)]
mod test {
    use crate::router::tree::art::node::bit_set::Bitset16;
    use crate::router::tree::art::node::NodeTrait;

    #[test]
    fn test_basic_mapping() {
        let mut mapping = super::IndexNode::<u8, 48, Bitset16<3>>::new();
        for i in 0..48 {
            mapping.add_child(i, i);
            assert_eq!(*mapping.find_child(i).unwrap(), i);
        }
        for i in 0..48 {
            assert_eq!(*mapping.find_child(i).unwrap(), i);
        }
        for i in 0..48 {
            assert_eq!(mapping.delete_child(i).unwrap(), i);
        }
        for i in 0..48 {
            assert!(mapping.find_child(i as u8).is_none());
        }
    }
}
