use crate::router::tree::art::node::{ NodeTrait};
use crate::router::tree::art::node::bit_array::BitArray;
use crate::router::tree::art::node::bit_set::BitsetTrait;
use crate::router::tree::art::node::index_node::IndexNode;
use crate::router::tree::art::utils::u8_keys_find_key_position;



pub struct KeyedNode<N, const WIDTH: usize, Bitset>
    where
        Bitset: BitsetTrait,
{
    pub(crate) keys: [u8; WIDTH],
    pub(crate) children: BitArray<N, WIDTH, Bitset>,
    pub(crate) num_children: u8,
}

impl<N, const WIDTH: usize, Bitset> Default for KeyedNode<N, WIDTH, Bitset>
    where
        Bitset: BitsetTrait,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N, const WIDTH: usize, Bitset> KeyedNode<N, WIDTH, Bitset>
    where
        Bitset: BitsetTrait,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            keys: [255; WIDTH],
            children: Default::default(),
            num_children: 0,
        }
    }

    pub(crate) fn from_indexed<const IDX_WIDTH: usize, FromBitset: BitsetTrait>(
        im: &mut IndexNode<N, IDX_WIDTH, FromBitset>,
    ) -> Self {
        let mut new_mapping = KeyedNode::new();
        im.num_children = 0;
        im.move_into::<WIDTH, KeyedNode<N, WIDTH, Bitset>>(&mut new_mapping);
        new_mapping
    }

    pub fn from_resized_grow<const OLD_WIDTH: usize, OldBitset: BitsetTrait>(
        km: &mut KeyedNode<N, OLD_WIDTH, OldBitset>,
    ) -> Self {
        assert!(WIDTH > OLD_WIDTH);
        let mut new = KeyedNode::new();

        for i in 0..OLD_WIDTH {
            new.keys[i] = km.keys[i];
            let stolen = km.children.erase(i);
            if let Some(n) = stolen {
                new.children.set(i, n);
            }
        }
        new.num_children = km.num_children;
        new
    }

    pub fn from_resized_shrink<const OLD_WIDTH: usize, OldBitset: BitsetTrait>(
        km: &mut KeyedNode<N, OLD_WIDTH, OldBitset>,
    ) -> Self {
        assert!(WIDTH < OLD_WIDTH);
        let mut new = KeyedNode::new();
        let mut cnt = 0;
        // Since we're smaller, we compact empty spots out.
        for i in 0..OLD_WIDTH {
            if km.children.check(i) {
                new.keys[cnt] = km.keys[i];
                let stolen = km.children.erase(i);
                if let Some(n) = stolen {
                    new.children.set(cnt, n);
                }
                cnt += 1;
            }
        }
        km.children.clear();
        new.num_children = km.num_children;
        km.num_children = 0;
        new
    }

    #[inline]
    pub(crate) fn iter(&self) -> impl Iterator<Item = (u8, &N)> {
        self.keys
            .iter()
            .enumerate()
            .filter(|p| self.children.check(p.0))
            .map(|p| (*p.1, self.children.get(p.0).unwrap()))
    }
}

impl<N, const WIDTH: usize, Bitset: BitsetTrait> NodeTrait<N>
for KeyedNode<N, WIDTH, Bitset>
{
    #[inline]
    fn add_child(&mut self, key: u8, node: N) {
        // Find an empty position by looking into the bitset.
        let idx = self.children.first_empty().unwrap_or_else(|| {
            // Invariant violated - upstream code should have checked for this.
            panic!(
                "No space left in bit array in KeyedMapping of size {}; num children: {} \
                 bitset used size: {}; capacity: {} storage size: {} bit width: {}",
                WIDTH,
                self.num_children,
                self.children.size(),
                self.children.bitset.capacity(),
                self.children.bitset.storage_width(),
                self.children.bitset.bit_width()
            )
        });
        assert!(idx < WIDTH);
        self.keys[idx] = key;
        self.children.set(idx, node);
        self.num_children += 1;
    }

    fn update_child(&mut self, key: u8, node: N) {
        *self.find_child_mut(key).unwrap() = node;
    }

    fn find_child(&self, key: u8) -> Option<&N> {
        let idx =
            u8_keys_find_key_position::<WIDTH, _>(key, &self.keys, &self.children.bitset)?;
        self.children.get(idx)
    }

    fn find_child_mut(&mut self, key: u8) -> Option<&mut N> {
        let idx =
            u8_keys_find_key_position::<WIDTH, _>(key, &self.keys, &self.children.bitset)?;
        self.children.get_mut(idx)
    }

    fn delete_child(&mut self, key: u8) -> Option<N> {
        // Find position of the key
        let idx =
            u8_keys_find_key_position::<WIDTH, _>(key, &self.keys, &self.children.bitset)?;
        let result = self.children.erase(idx);
        if result.is_some() {
            self.keys[idx] = 255;
            self.num_children -= 1;
        }
        // Return what we deleted, if any
        result
    }

    #[inline(always)]
    fn num_children(&self) -> usize {
        self.num_children as usize
    }

    #[inline]
    fn width(&self) -> usize {
        WIDTH
    }
}

impl<N, const WIDTH: usize, Bitset: BitsetTrait> Drop for KeyedNode<N, WIDTH, Bitset> {
    fn drop(&mut self) {
        self.children.clear();
        self.num_children = 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::router::tree::art::node::bit_set::Bitset8;
    use crate::router::tree::art::node::node::KeyedNode;
    use super::*;
    #[test]
    fn test_add_seek_delete() {
        let mut node = KeyedNode::<u8, 4, Bitset8<4>>::new();
        node.add_child(1, 1);
        node.add_child(2, 2);
        node.add_child(3, 3);
        node.add_child(4, 4);
        assert_eq!(node.num_children(), 4);
        assert_eq!(node.find_child(1), Some(&1));
        assert_eq!(node.find_child(2), Some(&2));
        assert_eq!(node.find_child(3), Some(&3));
        assert_eq!(node.find_child(4), Some(&4));
        assert_eq!(node.find_child(5), None);
        assert_eq!(node.find_child_mut(1), Some(&mut 1));
        assert_eq!(node.find_child_mut(2), Some(&mut 2));
        assert_eq!(node.find_child_mut(3), Some(&mut 3));
        assert_eq!(node.find_child_mut(4), Some(&mut 4));
        assert_eq!(node.find_child_mut(5), None);
        assert_eq!(node.delete_child(1), Some(1));
        assert_eq!(node.delete_child(2), Some(2));
        assert_eq!(node.delete_child(3), Some(3));
        assert_eq!(node.delete_child(4), Some(4));
        assert_eq!(node.delete_child(5), None);
        assert_eq!(node.num_children(), 0);
    }

    #[test]
    fn test_ff_regression() {
        // Test for scenario where children with '255' keys disappeared.
        let mut node = KeyedNode::<u8, 4, Bitset8<4>>::new();
        node.add_child(1, 1);
        node.add_child(21, 255);
        node.add_child(32, 3);
        node.add_child(34, 3);
        node.delete_child(3);
    }
}
