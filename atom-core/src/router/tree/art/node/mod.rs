pub mod keys;
pub mod partials;
pub mod bit_set;
pub mod bit_array;
pub mod node;
pub mod direct_node;
mod index_node;

pub struct BaseNode {

}

pub trait NodeTrait<N, const NUM_CHILDREN: usize> {
    fn add_child(&mut self, key: u8, node: N);
    fn update_child(&mut self, key: u8, node: N);
    fn find_child(&self, key: u8) -> Option<&N>;
    fn find_child_mut(&mut self, key: u8) -> Option<&mut N>;
    fn delete_child(&mut self, key: u8) -> Option<N>;
    fn num_children(&self) -> usize;
    fn width(&self) -> usize {
        NUM_CHILDREN
    }
}