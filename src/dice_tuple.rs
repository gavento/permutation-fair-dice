use itertools::Itertools;

use crate::{Word, FDTS};

#[derive(Clone, Eq, PartialEq)]
pub struct DiceTuple {
    pub word: Word,
    pub numbers: Word,
}

impl DiceTuple {
    pub fn as_string(&self) -> String {
        self.word.iter().map(|x| (x + 65u8) as char).collect()
    }

    pub fn from_numbers(f: &FDTS, numbers: &[u8]) -> Self {
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

    pub fn from_word(f: &FDTS, word: &[u8]) -> Self {
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

    pub fn from_string(f: &FDTS, word: &str) -> Self {
        let word = word
            .chars()
            .map(|c| {
                let c = c as usize;
                assert!(c >= 65 && c < 65 + 26, "invalid word character (only A-Z allowed)");
                (c - 65) as u8
            })
            .collect_vec();
        assert!(word.iter().all(|&d| (d as usize) < f.n()), "some dice character above 'A' + (number of dice - 1)");
        for (i, &s) in f.sizes.iter().enumerate() {
            assert_eq!(word.iter().filter(|&x| *x == i as u8).count(), s);
        }
        Self::from_word(f, &word)
    }
}

impl std::fmt::Debug for DiceTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_string())
    }
}
