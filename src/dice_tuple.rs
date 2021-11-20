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
}

impl std::fmt::Debug for DiceTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_string())
    }
}
