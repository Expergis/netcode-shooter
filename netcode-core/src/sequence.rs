//! Network sequence number tracking and wrap-around logic.

/// Determines if `sequence_a` is strictly newer than `sequence_b`,
/// handling 32-bit unsigned integer wrap-around.
#[inline]
pub fn is_sequence_newer(_sequence_a: u32, _sequence_b: u32) -> bool {
    let result = _sequence_a.wrapping_sub(_sequence_b);
    return result != 0 && result < (1 << 31)
}

/// Determines if `sequence_a` is less than or equal to `sequence_b`,
/// handling 32-bit unsigned integer wrap-around.
#[inline]
pub fn sequence_less_than_or_equal(_sequence_a: u32, _sequence_b: u32) -> bool {
    let result = _sequence_b.wrapping_sub(_sequence_a);
    return result != 0 && result < (1 << 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_newer_handles_wraparound() {
        assert!(is_sequence_newer(0, u32::MAX));
        assert!(is_sequence_newer(10, 5));
        assert!(!is_sequence_newer(5, 10));
        assert!(!is_sequence_newer(100, 100));
    }

    #[test]
    fn sequence_less_than_or_equal_handles_wraparound() {
        assert!(sequence_less_than_or_equal(5, 10));
        assert!(sequence_less_than_or_equal(u32::MAX, 0));
        assert!(!sequence_less_than_or_equal(10, 5));
        assert!(sequence_less_than_or_equal(100, 100));
    }
}
