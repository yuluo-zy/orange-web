use crate::router::tree::art::node::bit_array::BitArray;
use crate::router::tree::art::node::bit_set::{Bitset64, BitsetTrait};
use crate::router::tree::art::node::index_node::IndexNode;
use crate::router::tree::art::node::NodeTrait;

pub struct DirectNode<N> {
    pub(crate) children: BitArray<N, 256, Bitset64<4>>,  // 64 * 4
    num_children: usize,
}

impl<N> Default for DirectNode<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N> DirectNode<N> {
    pub fn new() -> Self {
        Self {
            children: BitArray::new(),
            num_children: 0,
        }
    }

    pub fn from_indexed<const WIDTH: usize, FromBitset: BitsetTrait>(
        im: &mut IndexNode<N, WIDTH, FromBitset>,
    ) -> Self {
        let mut new_mapping = DirectNode::<N>::new();
        im.num_children = 0;
        im.move_into::<WIDTH, DirectNode<N>>(&mut new_mapping);
        new_mapping
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (u8, &N)> {
        self.children.iter().map(|(key, node)| (key as u8, node))
    }
}

impl<N> NodeTrait<N> for DirectNode<N> {
    #[inline]
    fn add_child(&mut self, key: u8, node: N) {
        self.children.set(key as usize, node);
        self.num_children += 1;
    }

    fn update_child(&mut self, key: u8, node: N) {
        if let Some(n) = self.children.get_mut(key as usize) {
            *n = node;
        }
    }

    fn find_child(&self, key: u8) -> Option<&N> {
        self.children.get(key as usize)
    }

    fn find_child_mut(&mut self, key: u8) -> Option<&mut N> {
        self.children.get_mut(key as usize)
    }

    #[inline]
    fn delete_child(&mut self, key: u8) -> Option<N> {
        let n = self.children.erase(key as usize);
        if n.is_some() {
            self.num_children -= 1;
        }
        n
    }

    #[inline]
    fn num_children(&self) -> usize {
        self.num_children
    }

    #[inline]
    fn width(&self) -> u16 {
        256
    }
}

#[cfg(test)]
mod tests {
    use crate::router::tree::art::node::NodeTrait;

    #[test]
    fn direct_mapping_test() {
        let mut dm = super::DirectNode::new();
        for i in 0..255 {
            dm.add_child(i, i);
            assert_eq!(*dm.find_child(i).unwrap(), i);
            assert_eq!(dm.delete_child(i), Some(i));
            assert_eq!(dm.find_child(i), None);
        }
    }
}
