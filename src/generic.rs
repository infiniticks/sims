pub fn find_first_of(data: &[u8], searched: &[u8]) -> Option<usize> {
    for (i, v) in data.iter().enumerate() {
        if searched.contains(v) {
            return Some(i);
        }
    }
    None
}

pub fn find_first_not_of(data: &[u8], not_searched: &[u8]) -> Option<usize> {
    for (i, v) in data.iter().enumerate() {
        if !not_searched.contains(v) {
            return Some(i);
        }
    }
    None
}
