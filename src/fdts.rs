use std::io::{Read, Write};

use crate::{is_sorted, is_word_permutation_fair_up_to, DiceTuple, MappedFDTS, Word};
use itertools::Itertools;
use rustc_hash::FxHashSet as HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FDTS {
    pub sizes: Vec<usize>,
    pub total: usize,
    pub offsets: Vec<usize>,
    pub dice: Vec<DiceTuple>,
    pub prefixes: HashSet<Word>,
    pub fair_up_to: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct StoredFDTS {
    pub sizes: Vec<usize>,
    pub fair_up_to: usize,
    pub words: Vec<String>,
}

impl FDTS {
    /// Create empty FDTS with given sizes
    pub fn new_empty(sizes: &[usize]) -> Self {
        assert!(is_sorted(sizes));
        Self {
            total: sizes.iter().sum(),
            sizes: sizes.into(),
            offsets: sizes
                .iter()
                .scan(0, |s, v| {
                    *s += v;
                    Some(*s - v)
                })
                .collect(),
            dice: vec![],
            prefixes: HashSet::default(),
            fair_up_to: 0,
        }
    }

    /// Create a FDTS with a single `size`-sided (necessarily fair) dice
    pub fn new_single(size: usize) -> Self {
        let mut f = FDTS::new_empty(&[size]);
        f.insert_dice_tuple(DiceTuple::from_word(&f, &vec![0u8; size]));
        f.fair_up_to = 1;
        f
    }

    pub fn write_json(&self, writer: impl Write) -> serde_json::Result<()> {
        let s = StoredFDTS {
            sizes: self.sizes.clone(),
            words: self.dice.iter().map(|d| d.as_string()).collect(),
            fair_up_to: self.fair_up_to,
        };
        serde_json::to_writer_pretty(writer, &s)
    }

    pub fn from_json(reader: impl Read) -> serde_json::Result<Self> {
        let s: StoredFDTS = serde_json::from_reader(reader)?;
        let mut f = FDTS::new_empty(&s.sizes);
        f.fair_up_to = s.fair_up_to;
        assert!(f.fair_up_to <= f.n());
        let values = (0..f.n() as u8).collect_vec();
        for w in s.words {
            let dt = DiceTuple::from_string(&f, &w);
            assert!(is_word_permutation_fair_up_to(&dt.word, &values, f.fair_up_to));
            f.insert_dice_tuple(dt)
        }
        Ok(f)
    }

    /// Add a dice tuple and all the prefixes
    pub fn insert_dice_tuple(&mut self, d: DiceTuple) {
        for i in 0..=self.total {
            self.prefixes.insert(d.word[0..i].into());
        }
        self.dice.push(d);
    }

    /// Number of dice in FDTS
    pub fn n(&self) -> usize {
        self.sizes.len()
    }

    /// Sizes as comma-sep string
    pub fn sizes_string(&self) -> String {
        format!("[{}]", self.sizes.iter().format(","))
    }

    /// Create a MappedFDTS wrapping this FDTS (borrows non-mutably)
    pub fn mapped_as<'a>(&'a self, positions: &[isize]) -> MappedFDTS<'a> {
        let map: Vec<_> = (0..self.n())
            .into_iter()
            .map(|x| positions.iter().position(|&i| i == x as isize).expect("invalid positions"))
            .collect();
        MappedFDTS::new(self, &map, positions.len())
    }
}

#[cfg(test)]
mod test {
    use crate::fdts::FDTS;
    use crate::DiceTuple;

    #[test]
    fn test_basic() {
        let f = FDTS::new_empty(&[2, 3, 4]);
        assert!(f.offsets == [0, 2, 5]);
        assert!(f.total == 9);
        let d1 = DiceTuple::from_numbers(&f, &[0, 5, 2, 3, 6, 1, 4, 7, 8]);
        let d2 = DiceTuple::from_word(&f, &[0, 2, 1, 1, 2, 0, 1, 2, 2]);
        assert!(d1 == d2);
        assert!(d1.as_string() == "ACBBCABCC")
    }
}
