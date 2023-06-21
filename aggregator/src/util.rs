use eth_types::Field;
use halo2_proofs::circuit::AssignedCell;

use crate::{DEFAULT_KECCAK_ROWS, NUM_ROUNDS};

use std::env::var;

pub(crate) fn capacity(num_rows: usize) -> Option<usize> {
    if num_rows > 0 {
        // Subtract two for unusable rows
        Some(num_rows / ((NUM_ROUNDS + 1) * get_num_rows_per_round()) - 2)
    } else {
        None
    }
}

pub(crate) fn get_num_rows_per_round() -> usize {
    var("KECCAK_ROWS")
        .unwrap_or_else(|_| format!("{DEFAULT_KECCAK_ROWS}"))
        .parse()
        .expect("Cannot parse KECCAK_ROWS env var as usize")
}

/// Return
/// - the indices of the rows that contain the input preimages
/// - the indices of the rows that contain the output digest
pub(crate) fn get_indices(preimages: &[Vec<u8>]) -> (Vec<usize>, Vec<usize>) {
    let mut preimage_indices = vec![];
    let mut digest_indices = vec![];
    let mut round_ctr = 0;

    for preimage in preimages.iter() {
        //  136 = 17 * 8 is the size in bits of each
        //  input chunk that can be processed by Keccak circuit using absorb
        //  each chunk of size 136 needs 300 Keccak circuit rows to prove
        //  which consists of 12 Keccak rows for each of 24 + 1 Keccak cicuit rounds
        //  digest only happens at the end of the last input chunk with
        //  4 Keccak circuit rounds, so 48 Keccak rows, and 300 - 48 = 256
        let num_rounds = 1 + preimage.len() / 136;
        let mut preimage_padded = preimage.clone();
        preimage_padded.resize(136 * num_rounds, 0);
        for (i, round) in preimage_padded.chunks(136).enumerate() {
            // indices for preimages
            for (j, _chunk) in round.chunks(8).into_iter().enumerate() {
                for k in 0..8 {
                    preimage_indices.push(round_ctr * 300 + j * 12 + k + 12)
                }
            }
            // indices for digests
            if i == num_rounds - 1 {
                for j in 0..4 {
                    for k in 0..8 {
                        digest_indices.push(round_ctr * 300 + j * 12 + k + 252)
                    }
                }
            }
            round_ctr += 1;
        }
    }

    debug_assert!(is_ascending(&preimage_indices));
    debug_assert!(is_ascending(&digest_indices));

    (preimage_indices, digest_indices)
}

#[inline]
// assert two cells have same value
// (NOT constraining equality in circuit)
pub(crate) fn assert_equal<F: Field>(a: &AssignedCell<F, F>, b: &AssignedCell<F, F>) {
    let mut t1 = F::default();
    let mut t2 = F::default();
    a.value().map(|f| t1 = *f);
    b.value().map(|f| t2 = *f);
    assert_eq!(t1, t2)
}

#[inline]
// assert that the slice is ascending
fn is_ascending(a: &[usize]) -> bool {
    a.windows(2).all(|w| w[0] <= w[1])
}
