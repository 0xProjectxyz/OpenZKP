use super::{
    bit_reverse::permute_index, depth_first::depth_first_recurse, fft_vec_recursive, get_twiddles,
    transpose_square_stretch,
};
use crate::{FieldLike, Pow, RefFieldLike};
use log::trace;
use rayon::prelude::*;
use std::cmp::max;

/// In-place FFT with permuted output.
///
/// Implement's the six step FFT in a cache-oblivious manner. Output is
/// permuted, which avoids the last permutations step.
pub(super) fn radix_sqrt<Field>(values: &mut [Field], root: &Field)
where
    Field: FieldLike + std::fmt::Debug + From<usize> + Send + Sync,
    for<'a> &'a Field: RefFieldLike<Field>,
{
    // Recurse by splitting along the square root
    // Round such that outer is larger.
    let length = values.len();
    let inner = 1_usize << (length.trailing_zeros() / 2);
    let outer = length / inner;
    let stretch = outer / inner;
    debug_assert!(outer == inner || outer == 2 * inner);
    debug_assert_eq!(outer * inner, length);

    // Prepare twiddles
    let twiddles = get_twiddles(outer);

    // 1. Transpose inner x inner x stretch square matrix
    transpose_square_stretch(values, inner, stretch);

    // 2. Apply inner FFTs contiguously
    // 2. Apply twiddle factors
    trace!(
        "Parallel {} x inner FFT size {} (with twiddles)",
        outer,
        inner
    );
    values
        .par_chunks_mut(outer)
        .enumerate()
        .for_each(|(j, row)| {
            fft_vec_recursive(row, &twiddles, 0, stretch, stretch);

            // TODO: Fix twiddles (maybe do them in step 5?)
            if j > 0 {
                let outer_twiddle = root.pow(j);
                let mut inner_twiddle = outer_twiddle.clone();
                for i in 1..inner {
                    let i = permute_index(inner, i);
                    row[i] *= &inner_twiddle;
                    inner_twiddle *= &outer_twiddle;
                }
            }
        });

    // 4. Transpose inner x inner x stretch square matrix
    transpose_square_stretch(values, inner, stretch);

    // 5 Apply outer FFTs contiguously
    trace!("Parallel {} x outer FFT size {}", outer, inner);
    values
        .par_chunks_mut(outer)
        .for_each(|row| fft_vec_recursive(row, &twiddles, 0, 1, 1));
}

#[cfg(test)]
mod tests {
    use super::{
        super::tests::{arb_vec, ref_fft_permuted},
        *,
    };
    use crate::{FieldElement, Root};
    use proptest::prelude::*;
    use std::cmp::{max, min};

    proptest! {

        #[test]
        fn test_radix_sqrt(values in arb_vec()) {
            let root = FieldElement::root(values.len()).unwrap();
            let mut expected = values.clone();
            ref_fft_permuted(&mut expected);
            let mut result = values.clone();
            radix_sqrt(&mut result, &root);
            prop_assert_eq!(result, expected);
        }
    }
}
