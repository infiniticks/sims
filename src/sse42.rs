#[cfg(target_arch = "x86")]
use core::arch::x86 as arch;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64 as arch;

#[derive(Debug)]
enum ByteSetError {
    LoadTooBig,
}

#[repr(align(16))]
struct AlignedArray([u8; ByteSet::max_size()]);

#[derive(Copy, Clone)]
struct ByteSet {
    bytes: arch::__m128i,
    len: i32,
}

impl ByteSet {
    const fn max_size() -> usize {
        16
    }

    #[inline]
    unsafe fn load_unchecked(v: &[u8]) -> Self {
        Self {
            bytes: arch::_mm_loadu_si128(std::mem::transmute(v.as_ptr())),
            len: ByteSet::max_size() as i32,
        }
    }

    #[inline]
    unsafe fn load_aligned_unchecked(v: &[u8]) -> Self {
        Self {
            bytes: arch::_mm_load_si128(std::mem::transmute(v.as_ptr())),
            len: ByteSet::max_size() as i32,
        }
    }

    #[inline]
    unsafe fn load_partial_unchecked(v: &[u8]) -> Self {
        let mut v16 = AlignedArray([0; ByteSet::max_size()]);
        let len = std::cmp::min(v.len(), ByteSet::max_size());
        std::ptr::copy_nonoverlapping(v.as_ptr(), v16.0.as_mut_ptr(), len);
        Self {
            bytes: arch::_mm_load_si128(std::mem::transmute(v16.0.as_ptr())),
            len: len as i32,
        }
    }

    #[inline]
    fn find_any_in(&self, data: &ByteSet) -> Option<usize> {
        let res = unsafe {
            arch::_mm_cmpestri(
                self.bytes,
                self.len,
                data.bytes,
                data.len,
                arch::_SIDD_CMP_EQUAL_ANY,
            )
        };
        if res < data.len {
            Some(res as usize)
        } else {
            None
        }
    }

    #[inline]
    fn find_any_not_in(&self, data: &ByteSet) -> Option<usize> {
        let res = unsafe {
            arch::_mm_cmpestri(
                self.bytes,
                self.len,
                data.bytes,
                data.len,
                arch::_SIDD_CMP_EQUAL_ANY | arch::_SIDD_NEGATIVE_POLARITY,
            )
        };
        if res < data.len {
            Some(res as usize)
        } else {
            None
        }
    }

    #[inline]
    fn shift_right(&mut self, count: usize) {
        // TODO: the intrinsics function uses a const param
        // Use a match for now, investigate later
        self.bytes = unsafe {
            match count {
                0 => self.bytes,
                1 => arch::_mm_srli_si128(self.bytes, 1),
                2 => arch::_mm_srli_si128(self.bytes, 2),
                3 => arch::_mm_srli_si128(self.bytes, 3),
                4 => arch::_mm_srli_si128(self.bytes, 4),
                5 => arch::_mm_srli_si128(self.bytes, 5),
                6 => arch::_mm_srli_si128(self.bytes, 6),
                7 => arch::_mm_srli_si128(self.bytes, 7),
                8 => arch::_mm_srli_si128(self.bytes, 8),
                9 => arch::_mm_srli_si128(self.bytes, 9),
                10 => arch::_mm_srli_si128(self.bytes, 10),
                11 => arch::_mm_srli_si128(self.bytes, 11),
                12 => arch::_mm_srli_si128(self.bytes, 12),
                13 => arch::_mm_srli_si128(self.bytes, 13),
                14 => arch::_mm_srli_si128(self.bytes, 14),
                15 => arch::_mm_srli_si128(self.bytes, 15),
                _ => arch::_mm_set1_epi8(0),
            }
        };
        self.len = std::cmp::max(0, self.len - count as i32);
    }

    #[inline]
    fn get_byte(&self, index: usize) -> Option<u8> {
        if index >= self.len as usize {
            return None;
        }
        let mut bytes = AlignedArray([0; ByteSet::max_size()]);
        unsafe {
            arch::_mm_storeu_si128(std::mem::transmute(bytes.0.as_mut_ptr()), self.bytes);
        }
        Some(bytes.0[index])
    }
}

impl TryFrom<&[u8]> for ByteSet {
    type Error = ByteSetError;
    #[inline]
    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        use std::cmp::Ordering;
        let len = v.len();
        match len.cmp(&ByteSet::max_size()) {
            Ordering::Equal => Ok(unsafe { Self::load_unchecked(v) }),
            Ordering::Less => Ok(unsafe { Self::load_partial_unchecked(v) }),
            Ordering::Greater => Err(ByteSetError::LoadTooBig),
        }
    }
}

#[inline]
fn find_with(data: &[u8], finder: impl Fn(/*data*/ ByteSet) -> Option<usize>) -> Option<usize> {
    let data_len = data.len();
    if data_len <= ByteSet::max_size() {
        let chunk = if data_len == ByteSet::max_size() {
            unsafe { ByteSet::load_unchecked(data) }
        } else {
            unsafe { ByteSet::load_partial_unchecked(data) }
        };
        return finder(chunk);
    }
    // Load a first chunk in an unaligned fashion
    let mut offset: usize = data.as_ptr().align_offset(ByteSet::max_size());
    if offset != 0 {
        let chunk = unsafe { ByteSet::load_partial_unchecked(&data[..offset]) };
        if let Some(index) = finder(chunk) {
            return Some(offset + index);
        }
    }
    // Loading the next full chunks in an aligned fashion
    while offset + ByteSet::max_size() <= data_len {
        let chunk =
            unsafe { ByteSet::load_aligned_unchecked(&data[offset..offset + ByteSet::max_size()]) };
        if let Some(index) = finder(chunk) {
            return Some(offset + index);
        }
        offset += ByteSet::max_size();
    }
    // Loading the last partial chunk
    if offset < data_len {
        let chunk = unsafe { ByteSet::load_partial_unchecked(&data[offset..]) };
        if let Some(index) = finder(chunk) {
            return Some(offset + index);
        }
    }
    None
}

fn find_first_of_small(data: &[u8], searched: &[u8]) -> Option<usize> {
    let searched: ByteSet = searched.try_into().unwrap();
    find_with(data, |data| searched.find_any_in(&data))
}

fn find_first_of_large(data: &[u8], searched: &[u8]) -> Option<usize> {
    let searched: smallvec::SmallVec<[ByteSet; 2]> = searched
        .chunks(ByteSet::max_size())
        .map(|chunk| chunk.try_into().unwrap())
        .collect();
    find_with(data, |data| {
        searched.iter().filter_map(|s| s.find_any_in(&data)).min()
    })
}

pub fn find_first_not_of_small(data: &[u8], searched: &[u8]) -> Option<usize> {
    let searched: ByteSet = searched.try_into().unwrap();
    find_with(data, |data| searched.find_any_not_in(&data))
}

pub fn find_first_not_of_large(data: &[u8], not_searched: &[u8]) -> Option<usize> {
    let not_searched_parts: smallvec::SmallVec<[ByteSet; 2]> = not_searched
        .chunks(ByteSet::max_size())
        .map(|chunk| chunk.try_into().unwrap())
        .collect();
    find_with(data, |mut data| {
        let mut shifted = 0;
        loop {
            let mut max = 0;
            for ns in &not_searched_parts {
                let res = ns.find_any_not_in(&data);
                let Some(found) = res else {
                    return None;
                };
                max = std::cmp::max(max, found);
            }
            if !not_searched.contains(&data.get_byte(max).unwrap()) {
                return Some(max + shifted);
            }
            data.shift_right(max + 1);
            shifted += max + 1;
        }
    })
}

#[target_feature(enable = "sse4.2")]
#[inline]
pub unsafe fn find_first_of(data: &[u8], searched: &[u8]) -> Option<usize> {
    if searched.len() <= ByteSet::max_size() {
        find_first_of_small(data, searched)
    } else {
        find_first_of_large(data, searched)
    }
}

#[target_feature(enable = "sse4.2")]
#[inline]
pub unsafe fn find_first_not_of(data: &[u8], not_searched: &[u8]) -> Option<usize> {
    if not_searched.len() <= ByteSet::max_size() {
        find_first_not_of_small(data, not_searched)
    } else {
        find_first_not_of_large(data, not_searched)
    }
}
