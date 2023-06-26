use std::cell::Cell;
use std::fmt;
use num_traits::PrimInt;
use crate::router::tree::art::node::bit_set::BitsetTrait;


const SPIN_LIMIT: u32 = 6;
const YIELD_LIMIT: u32 = 10;

pub(crate) struct Backoff {
    step: Cell<u32>,
}

pub const EMPTY_NODE_ERROR: &str = "parent_node is empty";

impl Backoff {
    #[inline]
    pub(crate) fn new() -> Self {
        Backoff { step: Cell::new(0) }
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn reset(&self) {
        self.step.set(0);
    }

    #[inline]
    pub(crate) fn spin(&self) {
        // 循环2^(min(self.step.get(), SPIN_LIMIT))次
        for _ in 0..1 << self.step.get().min(SPIN_LIMIT) {
            std::hint::spin_loop();
        }

        if self.step.get() <= SPIN_LIMIT {
            self.step.set(self.step.get() + 1);
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn snooze(&self) {
        if self.step.get() <= SPIN_LIMIT {
            for _ in 0..1 << self.step.get() {
                std::hint::spin_loop();
            }
        } else {
            std::thread::yield_now();
        }

        if self.step.get() <= YIELD_LIMIT {
            self.step.set(self.step.get() + 1);
        }
    }

    #[inline]
    pub(crate) fn is_completed(&self) -> bool {
        self.step.get() > YIELD_LIMIT
    }
}
impl fmt::Debug for Backoff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Backoff")
            .field("step", &self.step)
            .field("is_completed", &self.is_completed())
            .finish()
    }
}

impl Default for Backoff {
    fn default() -> Backoff {
        Backoff::new()
    }
}

#[derive(Debug)]
pub enum TreeError {
    VersionNotMatch,
    Locked,
    Oom,
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[inline]
fn x86_64_sse_seek_insert_pos_16(key: u8, keys: [u8; 16], num_children: usize) -> Option<usize> {
    use std::arch::x86_64::{
        __m128i, _mm_cmplt_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
    };

    let bitfield = unsafe {
        let cmp_vec = _mm_set1_epi8(key as i8);
        let cmp = _mm_cmplt_epi8(cmp_vec, _mm_loadu_si128(keys.as_ptr() as *const __m128i));
        let mask = (1 << num_children) - 1;
        _mm_movemask_epi8(cmp) & mask
    };

    if bitfield != 0 {
        let idx = bitfield.trailing_zeros() as usize;
        return Some(idx);
    }
    None
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[inline]
fn x86_64_sse_find_key_16_up_to(key: u8, keys: [u8; 16], num_children: usize) -> Option<usize> {
    use std::arch::x86_64::{
        __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
    };

    let bitfield = unsafe {
        let key_vec = _mm_set1_epi8(key as i8);
        let results = _mm_cmpeq_epi8(key_vec, _mm_loadu_si128(keys.as_ptr() as *const __m128i));
        // AVX512 has _mm_cmpeq_epi8_mask which can allow us to skip this step and go direct to a
        // bitmask from comparison results.
        // ... but that's stdsimd nightly only for now, and also not available on all processors.
        let mask = (1 << num_children) - 1;
        _mm_movemask_epi8(results) & mask
    };
    if bitfield != 0 {
        let idx = bitfield.trailing_zeros() as usize;
        return Some(idx);
    }
    None
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
#[inline]
fn x86_64_sse_find_key_16(key: u8, keys: [u8; 16], bitmask: u16) -> Option<usize> {
    use std::arch::x86_64::{
        __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8,
    };

    let bitfield = unsafe {
        let key_vec = _mm_set1_epi8(key as i8);
        let results = _mm_cmpeq_epi8(key_vec, _mm_loadu_si128(keys.as_ptr() as *const __m128i));
        // AVX512 has _mm_cmpeq_epi8_mask which can allow us to skip this step and go direct to a
        // bitmask from comparison results.
        // ... but that's stdsimd nightly only for now, and also not available on all processors.
        _mm_movemask_epi8(results) & bitmask as i32
    };
    if bitfield != 0 {
        let idx = bitfield.trailing_zeros() as usize;
        return Some(idx);
    }
    None
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn x86_64_sse_find_key_32_up_to(
    key: u8,
    keys: [u8; 32],
    num_children: usize,
) -> Option<usize> {
    use std::arch::x86_64::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8,
    };

    let bitfield = unsafe {
        let key_vec = _mm256_set1_epi8(key as i8);
        let results =
            _mm256_cmpeq_epi8(key_vec, _mm256_loadu_si256(keys.as_ptr() as *const __m256i));
        let mask: i64 = (1 << num_children) - 1;
        _mm256_movemask_epi8(results) as i64 & mask
    };

    if bitfield != 0 {
        let idx = bitfield.trailing_zeros() as usize;

        return Some(idx);
    }
    None
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn x86_64_sse_find_key_32(key: u8, keys: [u8; 32], bitmask: u32) -> Option<usize> {
    use std::arch::x86_64::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8,
    };

    let bitfield = unsafe {
        let key_vec = _mm256_set1_epi8(key as i8);
        let results =
            _mm256_cmpeq_epi8(key_vec, _mm256_loadu_si256(keys.as_ptr() as *const __m256i));
        _mm256_movemask_epi8(results) as i64 & bitmask as i64
    };

    if bitfield != 0 {
        let idx = bitfield.trailing_zeros() as usize;

        return Some(idx);
    }
    None
}


fn binary_find_key(key: u8, keys: &[u8], num_children: usize) -> Option<usize> {
    let mut left = 0;
    let mut right = num_children;
    while left < right {
        let mid = (left + right) / 2;
        match keys[mid].cmp(&key) {
            std::cmp::Ordering::Less => left = mid + 1,
            std::cmp::Ordering::Equal => return Some(mid),
            std::cmp::Ordering::Greater => right = mid,
        }
    }
    None
}

#[allow(unreachable_code)]
pub fn u8_keys_find_key_position_sorted<const WIDTH: usize>(
    key: u8,
    keys: &[u8],
    num_children: usize,
) -> Option<usize> {
    // Width 4 and under, just use linear search.
    if WIDTH <= 4 {
        return (0..num_children).find(|&i| keys[i] == key);
    }

    // SIMD optimized forms of 16
    if WIDTH == 16 {
        #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2"
        ))]
        {
            return x86_64_sse_find_key_16_up_to(key, keys.try_into().unwrap(), num_children);
        }
    }

    // SIMD AVX only optimized form of 32
    if WIDTH == 32 {
        #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2"
        ))]
        {
            return unsafe {
                x86_64_sse_find_key_32_up_to(key, keys.try_into().unwrap(), num_children)
            };
        }
    }

    // Fallback to binary search.
    binary_find_key(key, keys, num_children)
}

#[allow(unreachable_code)]
pub fn u8_keys_find_key_position<const WIDTH: usize, Bitset: BitsetTrait>(
    key: u8,
    keys: &[u8],
    children_bitmask: &Bitset,
) -> Option<usize> {
    // SIMD optimized forms of 16
    if WIDTH == 16 {
        #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2"
        ))]
        {
            // Special 0xff key is special
            let mask = if key == 255 {
                children_bitmask.as_bitmask() as u16
            } else {
                0xffff
            };
            return x86_64_sse_find_key_16(key, keys.try_into().unwrap(), mask);
        }

    }

    // SIMD optimized forms of 32
    if WIDTH == 32 {
        #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2"
        ))]
        {
            // Special 0xff key is special
            let mask = if key == 255 {
                children_bitmask.as_bitmask() as u32
            } else {
                0xffffffff
            };
            return unsafe { x86_64_sse_find_key_32(key, keys.try_into().unwrap(), mask) };
        }
    }

    // Fallback to linear search for anything else (which is just WIDTH == 4, or if we have no
    // SIMD support).
    for (i, k) in keys.iter().enumerate() {
        if key == 255 && !children_bitmask.check(i) {
            continue;
        }
        if *k == key {
            return Some(i);
        }
    }
    None
}

pub fn u8_keys_find_insert_position<const WIDTH: usize>(
    key: u8,
    keys: &[u8],
    num_children: usize,
) -> Option<usize> {
    if WIDTH == 16 {
        #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2"
        ))]
        {
            return x86_64_sse_seek_insert_pos_16(key, keys.try_into().unwrap(), num_children)
                .or(Some(num_children));
        }

    }

    // Fallback: use linear search to find the insertion point.
    (0..num_children)
        .rev()
        .find(|&i| key < keys[i])
        .or(Some(num_children))
}
