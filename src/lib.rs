//! Simplistic string search
//!
//! This crate implements simplistic searches of byte slices, that may be
//! accelerated via SIMD operations.
//!
//! Currently, only a generic and an SSE4.2-optimized versions are available.
//! Selection is done at runtime via feature detection.

mod generic;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse42;

/// Find the index of the first byte in a slice that is equal to an
/// element of another slice.
///
/// # Arguments
/// * `data` - The slice for the values to be searched in
/// * `searched` - The byte values that are searched for
///
/// # Examples
/// ```
/// assert_eq!(sims::find_first_of(b"abcdefg", b"ego"), Some(4));
/// assert_eq!(sims::find_first_of(b"abcdefg", b"xyz"), None);
/// ```
#[inline]
pub fn find_first_of(data: &[u8], searched: &[u8]) -> Option<usize> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("sse4.2") {
        return unsafe { sse42::find_first_of(data, searched) };
    }
    generic::find_first_of(data, searched)
}

/// Find the index of the first byte in a slice that is not equal to an
/// element of another slice.
///
/// # Arguments
/// * `data` - The slice for the values to be searched in
/// * `not_searched` - The byte values that are not searched for
///
/// # Examples
/// ```
/// assert_eq!(sims::find_first_not_of(b"abcdefg", b"abde"), Some(2));
/// assert_eq!(sims::find_first_not_of(b"abcdefg", b"abcdefg"), None);
/// ```
#[inline]
pub fn find_first_not_of(data: &[u8], not_searched: &[u8]) -> Option<usize> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("sse4.2") {
        return unsafe { sse42::find_first_not_of(data, not_searched) };
    }
    generic::find_first_not_of(data, not_searched)
}

#[cfg(test)]
mod test;
