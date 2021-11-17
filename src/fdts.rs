#[allow(unused_imports)]
use std::{
    cmp,
    collections::{HashMap, HashSet},
    iter::Enumerate,
};

use crate::DiceTuple;
use log::info;
use smallvec::SmallVec;
type Word = SmallVec<[u8; 64]>;
// Consider: type Word = Vec<u8>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MappedFDTS<'a> {
    pub fdts: &'a FDTS,
    pub map: Vec<usize>,
    pub back: Vec<Option<usize>>,
}

impl<'a> MappedFDTS<'a> {
    pub fn new(fdts: &'a FDTS, map: &[usize], range: usize) -> Self {
        assert!(map.len() == fdts.n());
        let mut back = vec![None; range];
        for (i, &m) in map.iter().enumerate() {
            back[m] = Some(i);
        }
        Self {
            fdts,
            map: map.into(),
            back,
        }
    }

    pub fn iterate_words(&'a self) -> impl Iterator<Item = Word> + 'a {
        self.fdts.dice.iter().map(move |d| {
            d.word
                .iter()
                .map(|x| self.map[*x as usize] as u8)
                .collect::<Word>()
        })
    }

    pub fn iterate_words_subset(&'a self, subset: &[usize]) -> impl Iterator<Item = Word> + 'a {
        // Which internal indices to keep
        let mut keep: Vec<bool> = (0..self.map.len())
            .map(|i| subset.contains(&self.map[i]))
            .collect();
        self.fdts.dice.iter().map(move |d| {
            d.word
                .iter()
                .filter(|&&x| keep[x as usize])
                .map(|&x| self.map[x as usize] as u8)
                .collect::<Word>()
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FDTS {
    pub sizes: Vec<usize>,
    pub total: usize,
    pub offsets: Vec<usize>,
    pub dice: Vec<DiceTuple>,
    pub prefixes: HashSet<Word>,
}

impl FDTS {
    pub fn new_empty(sizes: &[usize]) -> Self {
        Self {
            sizes: sizes.into(),
            total: sizes.iter().sum(),
            offsets: sizes
                .iter()
                .scan(0, |s, v| {
                    *s += v;
                    Some(*s - v)
                })
                .collect(),
            dice: vec![],
            prefixes: HashSet::new(),
        }
    }

    pub fn new_single(size: usize) -> Self {
        let mut f = FDTS::new_empty(&[size]);
        f.insert_dice(DiceTuple::from_word(&f, &vec![0u8; size]));
        f
    }

    pub fn new_combined(
        d1: MappedFDTS<'_>,
        d2: MappedFDTS<'_>,
        checking: &[MappedFDTS<'_>],
    ) -> Self {
        let n = cmp::max(d1.back.len(), d2.back.len());
        let sizes: Vec<usize> = (0..n)
            .map(|i| {
                d1.back[i]
                    .or(d2.back[i])
                    .expect("not all indices are mapped to")
            })
            .collect();
        let mut f = FDTS::new_empty(&sizes);

        let bin_indices = d1.map.iter().filter(|i| d2.map.contains(i));
        //info!("Combining {} and {} into {}, checking {}");
        let mut bins1 = HashMap::<Word, Vec<Word>>::new();

        let mut bins2 = HashMap::<Word, Vec<Word>>::new();

        f
    }

    pub fn insert_dice(&mut self, d: DiceTuple) {
        for i in 0..=self.total {
            self.prefixes.insert(d.word[0..i].into());
        }
        self.dice.push(d);
    }

    pub fn n(&self) -> usize {
        self.sizes.len()
    }
}

mod test {
    use crate::fdts::{MappedFDTS, FDTS};
    use crate::{DiceTuple, Word};

    #[test]
    fn test_basic() {
        let mut f = FDTS::new_empty(&[2, 3, 4]);
        assert!(f.offsets == [0, 2, 5]);
        assert!(f.total == 9);
        let d1 = DiceTuple::from_numbers(&f, &[0, 5, 2, 3, 6, 1, 4, 7, 8]);
        let d2 = DiceTuple::from_word(&f, &[0, 2, 1, 1, 2, 0, 1, 2, 2]);
        assert!(d1 == d2);
        assert!(d1.as_string() == "ACBBCABCC")
    }

    #[test]
    fn test_mapped() {
        let mut f = FDTS::new_empty(&[2, 3, 2]);
        f.insert_dice(DiceTuple::from_word(&f, &[1, 2, 0, 1, 1, 0, 2]));

        let mf: MappedFDTS = MappedFDTS::new(&f, &[0, 2, 3], 4);
        assert_eq!(
            mf.iterate_words().collect::<Vec<_>>(),
            &[Word::from_slice(&[2, 3, 0, 2, 2, 0, 3])]
        );
        assert_eq!(
            mf.iterate_words_subset(&[3]).collect::<Vec<_>>(),
            &[Word::from_slice(&[3, 3])]
        );
        assert_eq!(
            mf.iterate_words_subset(&[1]).collect::<Vec<_>>(),
            &[Word::from_slice(&[])]
        );
        assert_eq!(
            mf.iterate_words_subset(&[0, 1, 2]).collect::<Vec<_>>(),
            &[Word::from_slice(&[2, 0, 2, 2, 0])]
        );
    }
}
