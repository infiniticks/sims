mod generic;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse42;

#[inline]
pub fn find_first_of(data: &[u8], searched: &[u8]) -> Option<usize> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("sse4.2") {
        return unsafe { sse42::find_first_of(data, searched) };
    }
    generic::find_first_of(data, searched)
}

#[inline]
pub fn find_first_not_of(data: &[u8], searched: &[u8]) -> Option<usize> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("sse4.2") {
        return unsafe { sse42::find_first_not_of(data, searched) };
    }
    generic::find_first_not_of(data, searched)
}

#[cfg(test)]
mod test;
