use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::Instant;

use crate::{subset_word, DiceTuple, Word};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use log::info;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

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
        let keep: Vec<bool> = (0..self.map.len())
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

    pub fn subset_word_in_prefixes(&self, word: &[u8]) -> bool {
        let bword: Word = word
            .iter()
            .filter_map(|&d| self.back[d as usize])
            .map(|x| x as u8)
            .collect();
        self.fdts.prefixes.contains(&bword)
    }

    pub fn sizes_string(&self) -> String {
        self.back
            .iter()
            .map(|b| {
                if let Some(i) = *b {
                    self.fdts.sizes[i].to_string()
                } else {
                    "_".into()
                }
            })
            .join(",")
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
        assert_eq!(d1.back.len(), d2.back.len());
        let sizes: Vec<usize> = (0..d1.back.len())
            .map(|i| {
                if let Some(i1) = d1.back[i] {
                    d1.fdts.sizes[i1]
                } else {
                    d2.fdts.sizes[d2.back[i].expect("not all indices are mapped to")]
                }
            })
            .collect();

        let mut f = FDTS::new_empty(&sizes);

        let bin_indices: Vec<_> = d1
            .map
            .iter()
            .cloned()
            .filter(|i| d2.map.contains(i))
            .collect();
        //info!("Combining {} and {} into {}, checking {}");

        let mut bins1 = HashMap::<Word, Vec<Word>>::new();
        for w in d1.iterate_words() {
            bins1
                .entry(subset_word(&w, &bin_indices))
                .or_default()
                .push(w);
        }

        let mut bins2 = HashMap::<Word, Vec<Word>>::new();
        for w in d2.iterate_words() {
            bins2
                .entry(subset_word(&w, &bin_indices))
                .or_default()
                .push(w);
        }

        let common_keys = bins1
            .keys()
            .filter(|&bw| bins2.contains_key(bw))
            .collect_vec();
        let total_pairs: usize = common_keys
            .iter()
            .map(|&bw| bins1[bw].len() * bins2[bw].len())
            .sum();

        let mut key_w1_pairs = vec![];
        for c in &common_keys {
            for w1 in &bins1[*c] {
                key_w1_pairs.push((c, w1));
            }
        }

        info!("Combining {} and {} dice with common positions {:?} and {} bins, interleaving total {} dice pairs", d1.fdts.dice.len(), d2.fdts.dice.len(), &bin_indices, common_keys.len(), total_pairs);
        let bar = ProgressBar::new(total_pairs as u64);
        bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "combining: {percent}%|{wide_bar}| {pos}/{len} pairs [{elapsed}<{eta}] {msg}",
                )
                .progress_chars("##-"),
        );
        let candidates = Mutex::new(0usize);
        let res = Mutex::new(vec![]);
        let t0 = Instant::now();

        key_w1_pairs.par_iter().for_each(|(&bw, w1)| {
            let mut local_c = 0;
            let mut local_res = Vec::new();
            for w2 in &bins2[bw] {
                for wi in f.interleave_words(w1, w2, &checking, &bin_indices) {
                    let dice = DiceTuple::from_word(&f, &wi);
                    local_c += 1;
                    if f.is_dice_fair(&dice) {
                        local_res.push(dice);
                    }
                }
            }
            let mut c = candidates.lock().unwrap();
            let mut r = res.lock().unwrap();
            *c += local_c;
            r.extend_from_slice(&mut local_res);
            bar.inc((bins2[bw].len()) as u64);
            bar.set_message(format!(
                "{} results, {} candidates, {:.2} cands/s",
                r.len(),
                *c,
                (*c as f64) / t0.elapsed().as_secs_f64()
            ));
        });

        bar.finish();
        for rd in res.into_inner().unwrap() {
            f.insert_dice(rd);
        }

        info!(
            "Created FDTS {:?} with {} fair DiceTuples ({} prefixes)",
            &f.sizes,
            f.dice.len(),
            f.prefixes.len(),
        );

        f
    }

    pub fn mapped_as<'a>(&'a self, positions: &[isize]) -> MappedFDTS<'a> {
        let map: Vec<_> = (0..self.n())
            .into_iter()
            .map(|x| {
                positions
                    .iter()
                    .position(|&i| i == x as isize)
                    .expect("invalid position map")
            })
            .collect();
        MappedFDTS::new(self, &map, positions.len())
    }

    fn _rec_interleave_words(
        &self,
        out: &mut Word,
        w1: &[u8],
        w2: &[u8],
        checking: &[MappedFDTS],
        common_dice: &[usize],
        res: &mut Vec<Word>,
    ) {
        for c in checking {
            if !c.subset_word_in_prefixes(out) {
                return;
            }
        }
        // TODO: Use checking
        if w1.is_empty() && w2.is_empty() {
            res.push(out.clone());
            return;
        }
        if w1.is_empty() {
            out.extend_from_slice(w2);
            self._rec_interleave_words(out, &[], &[], checking, common_dice, res);
            out.truncate(out.len() - w2.len());
            return;
        }
        if w2.is_empty() {
            out.extend_from_slice(w1);
            self._rec_interleave_words(out, &[], &[], checking, common_dice, res);
            out.truncate(out.len() - w1.len());
            return;
        }
        if w1[0] == w2[0] {
            out.push(w1[0]);
            self._rec_interleave_words(out, &w1[1..], &w2[1..], checking, common_dice, res);
            out.pop();
            return;
        }
        if common_dice.contains(&(w1[0] as usize)) {
            debug_assert!(!common_dice.contains(&(w2[0] as usize)));
            out.push(w2[0]);
            self._rec_interleave_words(out, &w1, &w2[1..], checking, common_dice, res);
            out.pop();
            return;
        }
        if common_dice.contains(&(w2[0] as usize)) {
            debug_assert!(!common_dice.contains(&(w1[0] as usize)));
            out.push(w1[0]);
            self._rec_interleave_words(out, &w1[1..], &w2, checking, common_dice, res);
            out.pop();
            return;
        }
        out.push(w1[0]);
        self._rec_interleave_words(out, &w1[1..], &w2, checking, common_dice, res);
        out.pop();
        out.push(w2[0]);
        self._rec_interleave_words(out, &w1, &w2[1..], checking, common_dice, res);
        out.pop();
    }

    fn interleave_words(
        &self,
        w1: &Word,
        w2: &Word,
        checking: &[MappedFDTS],
        common_dice: &[usize],
    ) -> Vec<Word> {
        let mut res = Vec::new();
        let mut buf = Word::new();
        self._rec_interleave_words(&mut buf, w1, w2, checking, common_dice, &mut res);
        res
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

    pub fn sizes_string(&self) -> String {
        format!("{}", self.sizes.iter().format(","))
    }
}

impl FDTS {
    fn _rec_is_dice_fair(
        &self,
        d: &DiceTuple,
        drawn: &mut Word,
        dice_no: usize,
        counters: &mut HashMap<Vec<bool>, usize>,
    ) {
        if dice_no == self.n() {
            let mut key = Vec::new();
            for i1 in 0..self.n() - 1 {
                for i2 in i1 + 1..self.n() {
                    key.push(drawn[i1] < drawn[i2]);
                }
            }
            *counters.entry(key).or_default() += 1;
        } else {
            let o = self.offsets[dice_no];
            for &roll in &d.numbers[o..o + self.sizes[dice_no]] {
                drawn.push(roll);
                self._rec_is_dice_fair(d, drawn, dice_no + 1, counters);
                drawn.pop();
            }
        }
    }

    pub fn is_dice_fair(&self, d: &DiceTuple) -> bool {
        let perms = (1..=self.n()).product();
        let mut counters = HashMap::new();
        self._rec_is_dice_fair(d, &mut Word::new(), 0, &mut counters);
        let vals: Vec<usize> = counters.into_values().collect();
        if vals.len() != perms {
            return false;
        }
        vals.iter().all(|&v| v == vals[0])
    }
}

mod test {
    #![allow(unused_imports)]

    use crate::fdts::{MappedFDTS, FDTS};
    use crate::{DiceTuple, Word};

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

    #[test]
    fn test_fair() {
        let f = FDTS::new_empty(&[2, 3]);
        let d1 = DiceTuple::from_word(&f, &[0, 1, 1, 1, 0]);
        assert!(f.is_dice_fair(&d1));
        let d2 = DiceTuple::from_word(&f, &[1, 1, 0, 0, 1]);
        assert!(!f.is_dice_fair(&d2));
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

        let mf2 = f.mapped_as(&[0, -1, 1, 2]);
        assert_eq!(
            mf2.iterate_words().collect::<Vec<_>>(),
            &[Word::from_slice(&[2, 3, 0, 2, 2, 0, 3])]
        );
    }
}
