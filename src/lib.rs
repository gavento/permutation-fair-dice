mod dice_tuple;
mod fdts;
mod mapped_fdts;
mod permutations;

pub use dice_tuple::DiceTuple;
pub use fdts::FDTS;
pub use mapped_fdts::MappedFDTS;
pub use permutations::{count_permutation_occurences, is_word_permutation_fair};

type Word = smallvec::SmallVec<[u8; 64]>;
// Consider: type Word = Vec<u8>;

pub fn subset_word(w: &Word, subset: &[usize]) -> Word {
    w.iter().cloned().filter(|&x| subset.contains(&(x as usize))).collect()
}

pub fn is_sorted<T>(data: impl AsRef<[T]>) -> bool
where
    T: Ord,
{
    let data = data.as_ref();
    data.windows(2).all(|w| w[0] <= w[1])
}
