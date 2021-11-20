use itertools::Itertools;
use smallvec::{smallvec, SmallVec};

/// Count the occurences of `permutation` as a subsequence of `word`.
/// Assumes `permutation` to contain every number at most once.
pub fn count_permutation_occurences<'a, A, B>(permutation: A, word: B) -> u64
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

/// Check if all permutations of `values` (unique numbers) are present in `word` equally often.
/// Note: `values` does not need to be 0..n, any set is supported.
/// Complexity: factorial(len(values)) * (len(values) + max(values))
pub fn is_word_permutation_fair<A, B>(word: A, values: B) -> bool
where
    A: AsRef<[u8]>,
    B: AsRef<[u8]>,
{
    let word: &[u8] = word.as_ref();
    let values: &[u8] = values.as_ref();
    assert!(!values.is_empty());
    let mut count = None;
    for p in values.iter().cloned().permutations(values.len()) {
        let c = count_permutation_occurences(&p, word);
        match count {
            None => count = Some(c),
            Some(c0) if c != c0 => {
                return false;
            }
            _ => {}
        }
    }
    true
}

#[cfg(test)]
mod test {
    use crate::permutations::{count_permutation_occurences, is_word_permutation_fair};

    #[test]
    fn test_fairness() {
        assert_eq!(is_word_permutation_fair(&[], &[42, 43]), true);
        assert_eq!(is_word_permutation_fair(&[1, 1, 1, 1], &[2]), true);
        assert_eq!(is_word_permutation_fair(&[1, 1, 1, 1], &[1]), true);
        assert_eq!(is_word_permutation_fair(&[1, 1, 1, 0], &[0, 1]), false);
        assert_eq!(is_word_permutation_fair(&[0, 1, 2, 2, 1, 0], &[0, 1, 2]), false);
        assert_eq!(is_word_permutation_fair(&[3, 1, 3, 2, 2, 2, 1, 3, 3, 3, 3, 1, 2], &[1, 2, 3]), true);
        assert_eq!(
            is_word_permutation_fair(&[1, 3, 3, 2, 2, 2, 1, 3, 3, 3, 3, 1, 2], &[1, 2, 3]),
            false
        );
    }

    #[test]
    fn test_counting() {
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
