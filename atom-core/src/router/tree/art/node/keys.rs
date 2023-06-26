use std::mem;
use crate::router::tree::art::node::partials::{ArrPartial, Partial};


pub trait KeyTrait<Prefix>: Clone
    where
        Prefix: Partial,
{
    fn at(&self, pos: usize) -> u8;
    fn length_at(&self, at_depth: usize) -> usize;
    fn to_prefix(&self, at_depth: usize) -> Prefix;
    fn matches_slice(&self, slice: &[u8]) -> bool;
}

#[derive(Clone, Copy)]
pub struct ArrayKey<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> ArrayKey<N> {
    pub fn from_slice(data: &[u8]) -> Self {
        assert!(data.len() <= N, "data length is greater than array length");
        let mut arr = [0; N];
        arr[..data.len()].copy_from_slice(data);
        Self {
            data: arr,
            len: data.len(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        assert!(s.len() + 1 < N, "data length is greater than array length");
        let mut arr = [0; N];
        arr[..s.len()].copy_from_slice(s.as_bytes());
        Self {
            data: arr,
            len: s.len() + 1,
        }
    }

    pub fn from_string(s: &String) -> Self {
        assert!(s.len() + 1 < N, "data length is greater than array length");
        let mut arr = [0; N];
        arr[..s.len()].copy_from_slice(s.as_bytes());
        Self {
            data: arr,
            len: s.len() + 1,
        }
    }

    pub fn from_array<const S: usize>(arr: [u8; S]) -> Self {
        Self::from_slice(&arr)
    }
}

impl<const N: usize> KeyTrait<ArrPartial<N>> for ArrayKey<N> {
    fn at(&self, pos: usize) -> u8 {
        self.data[pos]
    }
    fn length_at(&self, at_depth: usize) -> usize {
        self.len - at_depth
    }
    fn to_prefix(&self, at_depth: usize) -> ArrPartial<N> {
        ArrPartial::from_slice(&self.data[at_depth..self.len])
    }

    fn matches_slice(&self, slice: &[u8]) -> bool {
        &self.data[..self.len] == slice
    }
}

impl<const N: usize> From<u8> for ArrayKey<N> {
    fn from(data: u8) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl<const N: usize> From<u16> for ArrayKey<N> {
    fn from(data: u16) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl<const N: usize> From<u32> for ArrayKey<N> {
    fn from(data: u32) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl<const N: usize> From<u64> for ArrayKey<N> {
    fn from(data: u64) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl<const N: usize> From<u128> for ArrayKey<N> {
    fn from(data: u128) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl<const N: usize> From<usize> for ArrayKey<N> {
    fn from(data: usize) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl<const N: usize> From<&str> for ArrayKey<N> {
    fn from(data: &str) -> Self {
        Self::from_str(data)
    }
}

impl<const N: usize> From<String> for ArrayKey<N> {
    fn from(data: String) -> Self {
        Self::from_string(&data)
    }
}
impl<const N: usize> From<&String> for ArrayKey<N> {
    fn from(data: &String) -> Self {
        Self::from_string(data)
    }
}

impl<const N: usize> From<i8> for ArrayKey<N> {
    fn from(val: i8) -> Self {
        let v: u8 = unsafe { mem::transmute(val) };
        let i = (v ^ 0x80) & 0x80;
        let j = i | (v & 0x7F);
        ArrayKey::from_slice(&j.to_be_bytes())
    }
}

impl<const N: usize> From<i16> for ArrayKey<N> {
    fn from(val: i16) -> Self {
        let v: u16 = unsafe { mem::transmute(val) };
        let xor = 1 << 15;
        let i = (v ^ xor) & xor;
        let j = i | (v & (u16::MAX >> 1));
        ArrayKey::from_slice(&j.to_be_bytes())
    }
}

impl<const N: usize> From<i32> for ArrayKey<N> {
    fn from(val: i32) -> Self {
        let v: u32 = unsafe { mem::transmute(val) };
        let xor = 1 << 31;
        let i = (v ^ xor) & xor;
        let j = i | (v & (u32::MAX >> 1));
        ArrayKey::from_slice(&j.to_be_bytes())
    }
}
impl<const N: usize> From<i64> for ArrayKey<N> {
    fn from(val: i64) -> Self {
        let v: u64 = unsafe { mem::transmute(val) };
        let xor = 1 << 63;
        let i = (v ^ xor) & xor;
        let j = i | (v & (u64::MAX >> 1));
        ArrayKey::from_slice(&j.to_be_bytes())
    }
}
impl<const N: usize> From<i128> for ArrayKey<N> {
    fn from(val: i128) -> Self {
        let v: u128 = unsafe { mem::transmute(val) };
        let xor = 1 << 127;
        let i = (v ^ xor) & xor;
        let j = i | (v & (u128::MAX >> 1));
        ArrayKey::from_slice(&j.to_be_bytes())
    }
}

impl<const N: usize> From<isize> for ArrayKey<N> {
    fn from(val: isize) -> Self {
        let v: usize = unsafe { mem::transmute(val) };
        let xor = 1 << 63;
        let i = (v ^ xor) & xor;
        let j = i | (v & (usize::MAX >> 1));
        ArrayKey::from_slice(&j.to_be_bytes())
    }
}
