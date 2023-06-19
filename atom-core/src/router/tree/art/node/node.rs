use std::alloc::{Layout, LayoutError};
use std::mem;

pub(crate) const NODE4TYPE: Result<Layout, LayoutError> = Layout::from_size_align(mem::size_of::<Node4>(),
                                                                                  mem::align_of::<Node4>(), );
pub(crate) const NODE16TYPE: Result<Layout, LayoutError> = Layout::from_size_align(mem::size_of::<Node16>(),
                                                                                   mem::align_of::<Node16>(), );
pub(crate) const NODE48TYPE: Result<Layout, LayoutError> = Layout::from_size_align(mem::size_of::<Node48>(),
                                                                                   mem::align_of::<Node48>(), );
pub(crate) const NODE256TYPE: Result<Layout, LayoutError> = Layout::from_size_align(mem::size_of::<Node256>(),
                                                                                    mem::align_of::<Node256>(), );

pub(crate) struct Node4 {}

pub(crate) struct Node16 {}

pub(crate) struct Node48 {}

pub(crate) struct Node256 {}