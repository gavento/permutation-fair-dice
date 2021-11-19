use std::{borrow::Borrow, ops::Deref};

use smallvec::{smallvec, SmallVec};

/// Count the occurences of `permutation` as a subsequence of `word`.
/// Assumes `permutation` to contain every number 0..(len-1) exactly once.
fn count_permutation_occurences<'a, A, B>(permutation: A, word: B) -> u64
where
    A: AsRef<[u8]>,
    B: IntoIterator<Item = &'a u8>,
{
    let permutation: &[u8] = permutation.as_ref();
    // required range
    let m = (*permutation.iter().max().expect("permutation needs to be non-empty") as usize) + 1;
    let mut inverse: SmallVec<[usize; 8]> = smallvec![usize::MAX; m];
    for i in 0..permutation.len() {
        inverse[permutation[i] as usize] = i;
    }
    let mut counts = vec![0; permutation.len()];
    for &w in word.into_iter() {
        let wu = w as usize;
        if wu >= inverse.len() || inverse[wu] == usize::MAX {
            continue;
        }
        let i = inverse[wu];
        if i == 0 {
            counts[i] += 1;
        } else {
            counts[i] += counts[i - 1];
        }
    }
    *counts.last().unwrap()
}

#[cfg(test)]
mod test {
    use crate::permutations::count_permutation_occurences;

    #[test]
    fn test_unit() {
        assert_eq!(count_permutation_occurences(&[1], &[0, 2]), 0);
        assert_eq!(count_permutation_occurences(&[1], &[1]), 1);
        assert_eq!(count_permutation_occurences(&[0, 1], &[0, 1, 2, 0, 1]), 3);
        assert_eq!(count_permutation_occurences(&[2], &vec![2; 42]), 42);
        assert_eq!(
            count_permutation_occurences(&[0, 3, 2, 1], &[0, 1, 2, 3, 0, 3, 4, 2, 1, 0, 0, 2, 1, 3]),
            9
        );
    }
}
