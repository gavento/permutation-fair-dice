#[allow(unused_imports)]
use std::iter::Enumerate;

pub mod fdts;

use smallvec::SmallVec;
type Word = SmallVec<[u8; 64]>;
// Consider: type Word = Vec<u8>;

pub fn subset_word(w : &Word, subset: &[usize]) -> Word {
    w.iter().cloned().filter(|&x|{subset.contains(&(x as usize))}).collect()
}


#[derive(Clone, Eq, PartialEq)]
pub struct DiceTuple {
    pub word: Word,
    pub numbers: Word,
}

impl DiceTuple {
    pub fn as_string(&self) -> String {
        self.word.iter().map(|x| (x + 65u8) as char).collect()
    }

    pub fn from_numbers(f: &fdts::FDTS, numbers: &[u8]) -> Self {
        assert_eq!(numbers.len(), f.total);
        let mut word: Word = Word::from_elem(0, f.total);
        for dn in 0..f.n() {
            for i in 0..f.sizes[dn] {
                word[numbers[f.offsets[dn] + i] as usize] = dn as u8;
            }
        }
        DiceTuple {
            word,
            numbers: numbers.into(),
        }
    }

    pub fn from_word(f: &fdts::FDTS, word: &[u8]) -> Self {
        assert_eq!(word.len(), f.total);
        let mut numbers: Word = Word::from_elem(0, f.total);
        let mut offsets = f.offsets.clone();
        for i in 0..f.total {
            let dn = word[i] as usize;
            numbers[offsets[dn]] = i as u8;
            offsets[dn] += 1;
        }
        DiceTuple {
            word: word.into(),
            numbers,
        }
    }
}

impl std::fmt::Debug for DiceTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_string())
    }
}
