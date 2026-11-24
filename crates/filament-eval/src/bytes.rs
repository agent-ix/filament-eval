// SPDX-License-Identifier: AGPL-3.0-or-later
//! The shared scalar operations for checked `Bytes` values (FR-091, FR-322).

use alloc::vec::Vec;

use filament_exact::Integer;

/// A failure while forming the result of byte concatenation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ConcatFailure {
    /// The combined length cannot be represented by this platform.
    #[error("the concatenated byte length exceeds the platform capacity")]
    LengthOverflow,
    /// The allocator could not reserve the combined byte length.
    #[error("could not reserve memory for the concatenated bytes")]
    AllocationFailed,
}

/// Return the number of octets in `bytes` as an unbounded integer.
#[must_use]
pub fn length(bytes: &[u8]) -> Integer {
    Integer::from(bytes.len())
}

/// Return the bytes of `left` followed by the bytes of `right`.
///
/// The output allocation is reserved fallibly, and its length is checked
/// before allocation. This operation imposes no byte limit of its own.
pub fn concat(left: &[u8], right: &[u8]) -> Result<Vec<u8>, ConcatFailure> {
    let length = left
        .len()
        .checked_add(right.len())
        .ok_or(ConcatFailure::LengthOverflow)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(length)
        .map_err(|_| ConcatFailure::AllocationFailed)?;
    result.extend_from_slice(left);
    result.extend_from_slice(right);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{concat, length};
    use filament_exact::Integer;
    use ix_trace_rs::trace;

    #[trace("TC-1231", "FR-091-AC-78")]
    #[test]
    fn bytes_length_counts_octets_including_the_empty_sequence() {
        assert_eq!(length(&[]), Integer::from(0_u64));
        assert_eq!(length(&[0xde, 0xad, 0xbe, 0xef]), Integer::from(4_u64));
    }

    #[trace("TC-1231", "FR-091-AC-78")]
    #[test]
    fn bytes_concat_is_left_to_right_associative_and_has_empty_identity() {
        let first = [0x00];
        let second = [0xff, 0x01];
        let third = [0x02];

        assert_eq!(concat(&[0x00, 0xff], &[0x01]).unwrap(), [0x00, 0xff, 0x01]);
        assert_eq!(concat(&[], &[0x00, 0xff]).unwrap(), [0x00, 0xff]);
        assert_eq!(concat(&[0x00, 0xff], &[]).unwrap(), [0x00, 0xff]);
        assert_eq!(
            concat(&concat(&first, &second).unwrap(), &third).unwrap(),
            concat(&first, &concat(&second, &third).unwrap()).unwrap(),
        );
    }
}
