pub fn encode(src: &[u8]) -> Vec<u8> {
    base91::slice_encode(src)
}

pub fn decode(src: &[u8]) -> Vec<u8> {
    // base91 helpers 直接返回 Vec<u8>，无错误类型
    base91::slice_decode(src)
}

/// Conservative upper bound of base91 expansion (approx <= 1.23x).
#[inline]
pub fn worst_case_len(n: usize) -> usize {
    (n * 123 + 99) / 100
}
