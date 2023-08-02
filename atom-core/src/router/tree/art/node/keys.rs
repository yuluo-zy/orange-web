use std::mem;
use std::ops::Index;
use log::info;

pub trait Partial {
    fn partial_before(&self, length: usize) -> Self;
    fn partial_from(&self, src_offset: usize, length: usize) -> Self;
    fn partial_after(&self, start: usize) -> Self;
    fn at(&self, pos: usize) -> u8;
    fn len(&self) -> usize;
    fn empty() -> Self;
    fn length_at(&self, at_depth: usize) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn prefix_length_common(&self, other: &Self) -> usize;
    fn prefix_length_key(&self, key: &Self, at_depth: usize) -> usize;
    fn prefix_length_slice(&self, slice: &[u8]) -> usize;
    fn to_slice(&self) -> &[u8];
}

#[derive(Clone, Debug, Eq)]
pub struct RawKey<const N: usize> {
    data: [u8; N],
    len: usize,
}
impl<const N: usize> PartialEq for RawKey<N> {
    fn eq(&self, other: &Self) -> bool {
        self.data[..self.len] == other.data[..other.len]
    }
}

impl<const SIZE: usize> RawKey<SIZE> {
    // todo: 优化复制问题
    pub fn from_slice(src: &[u8]) -> Self {
        assert!(src.len() <= SIZE);
        let mut data = [0; SIZE];
        data[..src.len()].clone_from_slice(src);
        Self {
            data,
            len: src.len(),
        }
    }

    pub fn to_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }


    pub fn from_str(s: &str) -> Self {
        assert!(s.len() + 1 < SIZE, "data length is greater than array length");
        let mut arr = [0; SIZE];
        arr[..s.len()].clone_from_slice(s.as_bytes());
        Self {
            data: arr,
            len: s.len() + 1,
        }
    }

    pub fn from_string(s: &String) -> Self {
        assert!(s.len() + 1 < SIZE, "data length is greater than array length");
        let mut arr = [0; SIZE];
        arr[..s.len()].clone_from_slice(s.as_bytes());
        Self {
            data: arr,
            len: s.len() + 1,
        }
    }

    pub fn from_array<const S: usize>(arr: [u8; S]) -> Self {
        Self::from_slice(&arr)
    }
}

impl<const SIZE: usize> Index<usize> for RawKey<SIZE> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        self.data.index(index)
    }
}

impl<const SIZE: usize> Partial for RawKey<SIZE> {
    fn partial_before(&self, length: usize) -> Self {
        assert!(length <= self.len);
        RawKey::from_slice(&self.data[..length])
    }

    fn partial_from(&self, src_offset: usize, length: usize) -> Self {
        assert!(src_offset + length <= self.len);
        RawKey::from_slice(&self.data[src_offset..src_offset + length])
    }

    fn partial_after(&self, start: usize) -> Self {
        assert!(start <= self.len);
        RawKey::from_slice(&self.data[start..self.len])
    }

    #[inline(always)]
    fn at(&self, pos: usize) -> u8 {
        assert!(pos < self.len);
        self.data[pos]
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    fn empty() -> Self {
        Self {
            data: [0; SIZE],
            len: 0,
        }
    }

    fn length_at(&self, at_depth: usize) -> usize {
        self.len - at_depth
    }

    #[inline(always)]
    fn prefix_length_common(&self, other: &Self) -> usize {
        self.prefix_length_slice(other.to_slice())
    }

    #[inline(always)]
    fn prefix_length_key(
        &self,
        key: &RawKey<SIZE>,
        at_depth: usize,
    ) -> usize {
        let mut len = key.length_at(at_depth);

        if self.len < len { len = self.len }
        if SIZE < len { len = SIZE }

        let mut idx = 0;
        while idx < len {
            if self.data[idx] != key.at(idx + at_depth) {
                break;
            }
            idx += 1;
        }
        idx
    }

    fn prefix_length_slice(&self, slice: &[u8]) -> usize {
        let mut len = slice.len();
        if self.len < len { len = self.len }
        if SIZE < len { len = SIZE }

        let mut idx = 0;
        while idx < len {
            if self.data[idx] != slice[idx] {
                break;
            }
            idx += 1;
        }
        idx
    }
    fn to_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

impl<const SIZE: usize> From<&[u8]> for RawKey<SIZE> {
    fn from(src: &[u8]) -> Self {
        Self::from_slice(src)
    }
}

impl<const N: usize> From<u8> for RawKey<N> {
    fn from(data: u8) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl<const N: usize> From<u16> for RawKey<N> {
    fn from(data: u16) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl<const N: usize> From<u32> for RawKey<N> {
    fn from(data: u32) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl<const N: usize> From<u64> for RawKey<N> {
    fn from(data: u64) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl<const N: usize> From<u128> for RawKey<N> {
    fn from(data: u128) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl<const N: usize> From<usize> for RawKey<N> {
    fn from(data: usize) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl<const N: usize> From<&str> for RawKey<N> {
    fn from(data: &str) -> Self {
        Self::from_str(data)
    }
}

impl<const N: usize> From<String> for RawKey<N> {
    fn from(data: String) -> Self {
        Self::from_string(&data)
    }
}
impl<const N: usize> From<&String> for RawKey<N> {
    fn from(data: &String) -> Self {
        Self::from_string(data)
    }
}

impl<const N: usize> From<i8> for RawKey<N> {
    fn from(val: i8) -> Self {
        let v: u8 = unsafe { mem::transmute(val) };
        let i = (v ^ 0x80) & 0x80;
        let j = i | (v & 0x7F);
        RawKey::from_slice(&j.to_be_bytes())
    }
}

impl<const N: usize> From<i16> for RawKey<N> {
    fn from(val: i16) -> Self {
        let v: u16 = unsafe { mem::transmute(val) };
        let xor = 1 << 15;
        let i = (v ^ xor) & xor;
        let j = i | (v & (u16::MAX >> 1));
        RawKey::from_slice(&j.to_be_bytes())
    }
}

impl<const N: usize> From<i32> for RawKey<N> {
    fn from(val: i32) -> Self {
        let v: u32 = unsafe { mem::transmute(val) };
        let xor = 1 << 31;
        let i = (v ^ xor) & xor;
        let j = i | (v & (u32::MAX >> 1));
        RawKey::from_slice(&j.to_be_bytes())
    }
}
impl<const N: usize> From<i64> for RawKey<N> {
    fn from(val: i64) -> Self {
        let v: u64 = unsafe { mem::transmute(val) };
        let xor = 1 << 63;
        let i = (v ^ xor) & xor;
        let j = i | (v & (u64::MAX >> 1));
        RawKey::from_slice(&j.to_be_bytes())
    }
}
impl<const N: usize> From<i128> for RawKey<N> {
    fn from(val: i128) -> Self {
        let v: u128 = unsafe { mem::transmute(val) };
        let xor = 1 << 127;
        let i = (v ^ xor) & xor;
        let j = i | (v & (u128::MAX >> 1));
        RawKey::from_slice(&j.to_be_bytes())
    }
}

impl<const N: usize> From<isize> for RawKey<N> {
    fn from(val: isize) -> Self {
        let v: usize = unsafe { mem::transmute(val) };
        let xor = 1 << 63;
        let i = (v ^ xor) & xor;
        let j = i | (v & (usize::MAX >> 1));
        RawKey::from_slice(&j.to_be_bytes())
    }
}


#[derive(Clone)]
pub struct VectorKey {
    data: Vec<u8>,
}

impl VectorKey {
    pub fn from_string(s: &String) -> Self {
        let mut data = Vec::with_capacity(s.len() + 1);
        data.extend_from_slice(s.as_bytes());
        data.push(0);
        Self { data }
    }

    pub fn from_str(s: &str) -> Self {
        let mut data = Vec::with_capacity(s.len() + 1);
        data.extend_from_slice(s.as_bytes());
        data.push(0);
        Self { data }
    }

    pub fn from_slice(data: &[u8]) -> Self {
        let data = Vec::from(data);
        Self { data }
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl Partial for VectorKey {
    fn partial_before(&self, length: usize) -> Self {
        let mut data = Vec::with_capacity(length);
        data.extend_from_slice(&self.data[..length]);
        Self { data }
    }

    fn partial_from(&self, src_offset: usize, length: usize) -> Self {
        let mut data = Vec::with_capacity(length);
        data.extend_from_slice(&self.data[src_offset..src_offset + length]);
        Self { data }
    }

    fn partial_after(&self, start: usize) -> Self {
        let mut data = Vec::with_capacity(self.data.len() - start);
        data.extend_from_slice(&self.data[start..]);
        Self { data }
    }

    fn at(&self, pos: usize) -> u8 {
        self.data[pos]
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn empty() -> Self {
        Self{
            data: vec![],
        }
    }

    fn length_at(&self, at_depth: usize) -> usize {
        self.data.len() - at_depth
    }

    fn is_empty(&self) -> bool {
       self.data.is_empty()
    }

    fn prefix_length_common(&self, other: &Self) -> usize {
        self.prefix_length_slice(other.to_slice())
    }

    fn prefix_length_key(
        &self,
        key: &VectorKey,
        at_depth: usize,
    ) -> usize {
        let mut len = key.length_at(at_depth);
        if self.data.len() < len {
            len = self.data.len();
        }
        let mut idx = 0;
        while idx < len {
            if self.data[idx] != key.at(idx + at_depth) {
                break;
            }
            idx += 1;
        }
        idx
    }

    fn prefix_length_slice(&self, slice: &[u8]) -> usize {
        let mut len = slice.len();
        if self.data.len() < len {
            len = self.data.len();
        }
        let mut idx = 0;
        while idx < len {
            if self.data[idx] != slice[idx] {
                break;
            }
            idx += 1;
        }
        idx
    }

    fn to_slice(&self) -> &[u8] {
        &self.data
    }
}


impl From<u8> for VectorKey {
    fn from(data: u8) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl From<u16> for VectorKey {
    fn from(data: u16) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl From<u32> for VectorKey {
    fn from(data: u32) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl From<u64> for VectorKey {
    fn from(data: u64) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl From<u128> for VectorKey {
    fn from(data: u128) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl From<usize> for VectorKey {
    fn from(data: usize) -> Self {
        Self::from_slice(&data.to_be_bytes())
    }
}

impl From<&str> for VectorKey {
    fn from(data: &str) -> Self {
        Self::from_str(data)
    }
}

impl From<String> for VectorKey {
    fn from(data: String) -> Self {
        Self::from_string(&data)
    }
}
impl From<&String> for VectorKey {
    fn from(data: &String) -> Self {
        Self::from_string(data)
    }
}

impl From<i8> for VectorKey {
    fn from(val: i8) -> Self {
        // flip upper bit of signed value to get comparable byte sequence:
        // -128 => 0
        // -127 => 1
        // 0 => 128
        // 1 => 129
        // 127 => 255
        let v: u8 = unsafe { mem::transmute(val) };
        // flip upper bit and set to 0 other bits:
        // (0000_1100 ^ 1000_0000) & 1000_0000 = 1000_0000
        // (1000_1100 ^ 1000_0000) & 1000_0000 = 0000_0000
        let i = (v ^ 0x80) & 0x80;
        // repair bits(except upper bit) of value:
        // self = -127
        // i = 0 (0b0000_0000)
        // v = 129 (0b1000_0001)
        // j = 0b0000_0000 | (0b1000_0001 & 0b0111_1111) = 0b0000_0000 | 0b0000_0001 = 0b0000_0001 = 1
        let j = i | (v & 0x7F);
        VectorKey::from_slice(&j.to_be_bytes())
    }
}

impl From<i16> for VectorKey {
    fn from(val: i16) -> Self {
        let v: u16 = unsafe { mem::transmute(val) };
        let xor = 1 << 15;
        let i = (v ^ xor) & xor;
        let j = i | (v & (u16::MAX >> 1));
        VectorKey::from_slice(&j.to_be_bytes())
    }
}

impl From<i32> for VectorKey {
    fn from(val: i32) -> Self {
        let v: u32 = unsafe { mem::transmute(val) };
        let xor = 1 << 31;
        let i = (v ^ xor) & xor;
        let j = i | (v & (u32::MAX >> 1));
        VectorKey::from_slice(&j.to_be_bytes())
    }
}
impl From<i64> for VectorKey {
    fn from(val: i64) -> Self {
        let v: u64 = unsafe { mem::transmute(val) };
        let xor = 1 << 63;
        let i = (v ^ xor) & xor;
        let j = i | (v & (u64::MAX >> 1));
        VectorKey::from_slice(&j.to_be_bytes())
    }
}
impl From<i128> for VectorKey {
    fn from(val: i128) -> Self {
        let v: u128 = unsafe { mem::transmute(val) };
        let xor = 1 << 127;
        let i = (v ^ xor) & xor;
        let j = i | (v & (u128::MAX >> 1));
        VectorKey::from_slice(&j.to_be_bytes())
    }
}

impl From<isize> for VectorKey {
    fn from(val: isize) -> Self {
        let v: usize = unsafe { mem::transmute(val) };
        let xor = 1 << 63;
        let i = (v ^ xor) & xor;
        let j = i | (v & (usize::MAX >> 1));
        VectorKey::from_slice(&j.to_be_bytes())
    }
}

