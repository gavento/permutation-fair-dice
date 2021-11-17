use std::collections::HashSet;

use smallvec::SmallVec;

type Word = SmallVec<[u8; 64]>;
// Consider: type Word = Vec<u8>;

#[derive(Debug, Clone, Eq, PartialEq)]
struct DiceTuple {
    pub word: Word,
    pub numbers: Word,
}

const UNUSED: usize = 1 << 31;

#[derive(Debug, Clone, Eq, PartialEq)]
struct MappedFDTS<'a> {
    pub fdts: &'a FDTS,
    pub map: Vec<usize>,
    pub back: Vec<usize>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct FDTS {
    sizes: Vec<usize>,
    total: usize,
    offsets: Vec<usize>,
    dice: Vec<DiceTuple>,
    prefixes: HashSet<Word>,
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
        let mut s = FDTS::new_empty(&[size]);
        s.insert_dice(s.dice_tuple_from_word(&vec![0u8; size]));
        s
    }

    pub fn new_combined(
        d1: MappedFDTS<'_>,
        d2: MappedFDTS<'_>,
        checking: &[MappedFDTS<'_>],
    ) -> Self {
        unimplemented!()
    }

    pub fn insert_dice(&mut self, d: DiceTuple) {
        for i in 0..=self.total {
            self.prefixes.insert(d.word[0..i].into());
        }
        self.dice.push(d);
    }

    pub fn dice_tuple_from_numbers(&self, numbers: &[u8]) -> DiceTuple {
        let mut word: Word = Word::from_elem(0, self.total);
        for dn in 0..self.n() {
            for i in 0..self.sizes[dn] {
                word[numbers[self.offsets[dn] + i] as usize] = dn as u8;
            }
        }
        DiceTuple { word, numbers: numbers.into() }
    }

    pub fn dice_tuple_from_word(&self, word: &[u8]) -> DiceTuple {
        let mut numbers: Word = Word::from_elem(0, self.total);
        let mut offsets = self.offsets.clone();
        for i in 0..self.total {
            let dn = word[i] as usize;
            numbers[offsets[dn]] = i as u8;
            offsets[dn] += 1;
        }
        DiceTuple { word: word.into(), numbers }
    }

    pub fn n(&self) -> usize {
        self.sizes.len()
    }
}

mod test {
    use crate::FDTS;

    #[test]
    fn test_basic() {
        let mut f = FDTS::new_empty(&[2,3,4]);
        assert!(f.offsets == [0,2,5]);
        assert!(f.total == 9);
        let d1 = f.dice_tuple_from_numbers(&[0,5,2,3,6,1,4,7,8]);
        let d2 = f.dice_tuple_from_word(&[0,2,1,1,2,0,1,2,2]);
        assert!(d1 == d2);
    }
}

fn main() {
    println!("Hello, world!");
}
