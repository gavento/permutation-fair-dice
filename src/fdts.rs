use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::Instant;

use crate::{is_sorted, subset_word, DiceTuple, Word};
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
        assert!(is_sorted(map));
        assert!(is_sorted(&back.iter().filter_map(|&x| x).collect_vec()));
        Self {
            fdts,
            map: map.into(),
            back,
        }
    }

    pub fn iterate_words(&'a self) -> impl Iterator<Item = Word> + 'a {
        self.fdts
            .dice
            .iter()
            .map(move |d| d.word.iter().map(|x| self.map[*x as usize] as u8).collect::<Word>())
    }

    pub fn iterate_words_subset(&'a self, subset: &[usize]) -> impl Iterator<Item = Word> + 'a {
        // Which internal indices to keep
        let keep: Vec<bool> = (0..self.map.len()).map(|i| subset.contains(&self.map[i])).collect();
        self.fdts.dice.iter().map(move |d| {
            d.word
                .iter()
                .filter(|&&x| keep[x as usize])
                .map(|&x| self.map[x as usize] as u8)
                .collect::<Word>()
        })
    }

    pub fn subset_word_in_prefixes(&self, word: &[u8]) -> bool {
        let bword: Word = word.iter().filter_map(|&d| self.back[d as usize]).map(|x| x as u8).collect();
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

    pub fn is_compatible_with(&self, other: &MappedFDTS) -> bool {
        if self.back.len() != other.back.len() {
            return false;
        }
        for (&b1, &b2) in self.back.iter().zip(other.back.iter()) {
            if b1.is_some() && b2.is_some() {
                if self.fdts.sizes[b1.unwrap()] != other.fdts.sizes[b2.unwrap()] {
                    return false;
                }
            }
        }
        true
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
            prefixes: HashSet::new(),
        }
    }

    pub fn new_single(size: usize) -> Self {
        let mut f = FDTS::new_empty(&[size]);
        f.insert_dice(DiceTuple::from_word(&f, &vec![0u8; size]));
        f
    }

    pub fn new_combined(d1: MappedFDTS<'_>, d2: MappedFDTS<'_>, checking: &[MappedFDTS<'_>]) -> Self {
        assert!(d1.is_compatible_with(&d2));
        for c in checking {
            assert!(d1.is_compatible_with(c));
        }

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

        let bin_indices: Vec<_> = d1.map.iter().cloned().filter(|i| d2.map.contains(i)).collect();
        info!(
            "Combining ({}) and ({}) into ({}), checking {:?}",
            d1.sizes_string(),
            d2.sizes_string(),
            f.sizes_string(),
            checking.iter().map(|c| { c.sizes_string() }).collect_vec()
        );

        let mut bins1 = HashMap::<Word, Vec<Word>>::new();
        for w in d1.iterate_words() {
            bins1.entry(subset_word(&w, &bin_indices)).or_default().push(w);
        }

        let mut bins2 = HashMap::<Word, Vec<Word>>::new();
        for w in d2.iterate_words() {
            bins2.entry(subset_word(&w, &bin_indices)).or_default().push(w);
        }

        let common_keys = bins1.keys().filter(|&bw| bins2.contains_key(bw)).collect_vec();
        let total_pairs: usize = common_keys.iter().map(|&bw| bins1[bw].len() * bins2[bw].len()).sum();

        let mut key_w1_pairs = vec![];
        for c in &common_keys {
            for w1 in &bins1[*c] {
                key_w1_pairs.push((c, w1));
            }
        }

        info!(
            " .. combining {} and {} dice with common positions {:?} and {} bins, interleaving total {} dice pairs",
            d1.fdts.dice.len(),
            d2.fdts.dice.len(),
            &bin_indices,
            common_keys.len(),
            total_pairs
        );
        let bar = ProgressBar::new(total_pairs as u64);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("combining: {percent}%|{wide_bar}| {pos}/{len} pairs [{elapsed}<{eta}] {msg}")
                .progress_chars("##-"),
        );
        let candidates = Mutex::new(0usize);
        let res = Mutex::new(vec![]);
        let t0 = Instant::now();

        key_w1_pairs.par_iter().for_each(|(&bw, w1)| {
            for w2 in &bins2[bw] {
                let mut local_c = 0;
                let mut local_res = Vec::new();
                for wi in f.interleave_words(w1, w2, &checking, &bin_indices, true) {
                    let dice = DiceTuple::from_word(&f, &wi);
                    local_c += 1;
                    debug_assert_eq!(f.is_dice_fair(&dice), is_dice_word_fair(&wi, 14));
                    if is_dice_word_fair(&wi, 14) {
                        local_res.push(dice);
                    }
                }
                let mut c = candidates.lock().unwrap();
                let mut r = res.lock().unwrap();
                *c += local_c;
                r.extend_from_slice(&mut local_res);
                //bar.inc((bins2[bw].len()) as u64);
                bar.inc(1);
                bar.set_message(format!(
                    "{} results, {} candidates, {:.2} cands/s",
                    r.len(),
                    *c,
                    (*c as f64) / t0.elapsed().as_secs_f64()
                ));
            }
        });

        bar.finish();
        for rd in res.into_inner().unwrap() {
            f.insert_dice(rd);
        }

        info!(
            " .. created FDTS {:?} with {} fair DiceTuples ({} prefixes)",
            &f.sizes,
            f.dice.len(),
            f.prefixes.len(),
        );

        f
    }

    pub fn mapped_as<'a>(&'a self, positions: &[isize]) -> MappedFDTS<'a> {
        let map: Vec<_> = (0..self.n())
            .into_iter()
            .map(|x| positions.iter().position(|&i| i == x as isize).expect("invalid position map"))
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

    fn _push_rec_lex(
        &self,
        out: &mut Word,
        c: u8,
        w1x: &[u8],
        w2x: &[u8],
        checking: &[MappedFDTS],
        c_d: &[usize],
        res: &mut Vec<Word>,
        cg: &[bool],
        icg: &[usize],
    ) {
        if !cg[c as usize] {
            return;
        }
        out.push(c);
        let im = icg[c as usize];
        if !cg[im] {
            let mut cg2: Vec<bool> = cg.into();
            cg2[im] = true;
            self._rec_interleave_words_lex(out, w1x, w2x, checking, c_d, res, &cg2, icg);
        } else {
            self._rec_interleave_words_lex(out, w1x, w2x, checking, c_d, res, cg, icg);
        }
        out.pop();
    }

    fn _rec_interleave_words_lex(
        &self,
        out: &mut Word,
        w1: &[u8],
        w2: &[u8],
        checking: &[MappedFDTS],
        common_dice: &[usize],
        res: &mut Vec<Word>,
        can_go: &[bool],
        implies_can_go: &[usize],
    ) {
        for c in checking {
            if !c.subset_word_in_prefixes(out) {
                return;
            }
        }
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
        if can_go.iter().all(|&x| x) {
            return self._rec_interleave_words(out, w1, w2, checking, common_dice, res);
        }

        if w1[0] == w2[0] {
            debug_assert!(common_dice.contains(&(w1[0] as usize)));
            self._push_rec_lex(out, w1[0], &w1[1..], &w2[1..], checking, common_dice, res, can_go, implies_can_go);
            return;
        }
        if common_dice.contains(&(w1[0] as usize)) {
            debug_assert!(!common_dice.contains(&(w2[0] as usize)));
            self._push_rec_lex(out, w2[0], &w1, &w2[1..], checking, common_dice, res, can_go, implies_can_go);
            return;
        }
        if common_dice.contains(&(w2[0] as usize)) {
            debug_assert!(!common_dice.contains(&(w1[0] as usize)));
            self._push_rec_lex(out, w1[0], &w1[1..], &w2, checking, common_dice, res, can_go, implies_can_go);
            return;
        }
        self._push_rec_lex(out, w1[0], &w1[1..], &w2, checking, common_dice, res, can_go, implies_can_go);
        self._push_rec_lex(out, w2[0], &w1, &w2[1..], checking, common_dice, res, can_go, implies_can_go);
    }

    fn interleave_words(
        &self,
        w1: &Word,
        w2: &Word,
        checking: &[MappedFDTS],
        common_dice: &[usize],
        same_lexicographic: bool,
    ) -> Vec<Word> {
        let mut res = Vec::new();
        let mut buf = Word::new();
        if same_lexicographic {
            let mut size_groups = HashMap::new();
            let mut can_go = vec![false; self.n()];
            let mut implies_can_go = (0..self.n()).collect_vec();
            for (i, &s) in self.sizes.iter().enumerate() {
                size_groups.entry(s).or_insert(vec![]).push(i);
            }
            for (_, is) in size_groups {
                can_go[is[0]] = true;
                for wi in is.as_slice().windows(2) {
                    implies_can_go[wi[0]] = wi[1];
                }
            }
            self._rec_interleave_words_lex(&mut buf, w1, w2, checking, common_dice, &mut res, &can_go, &implies_can_go);
        } else {
            self._rec_interleave_words(&mut buf, w1, w2, checking, common_dice, &mut res);
        }
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
    fn _rec_is_dice_fair(&self, d: &DiceTuple, drawn: &mut Word, dice_no: usize, counters: &mut HashMap<Vec<bool>, usize>) {
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

thread_local! {
    static SUBPERM_TABLES: RefCell<Vec<HashMap<Word, usize>>> = RefCell::new(vec![HashMap::new(); 16]);
}

// Counts occurences of the permutation (n-1, .., 0) in the given word
fn rec_check_perm(n: u8, word: &[u8], cache_limit: usize) -> usize {
    if n == 0 {
        return 1;
    }
    if word.is_empty() {
        return 0;
    }

    if word.len() < cache_limit {
        let cached = SUBPERM_TABLES.with(|st| st.borrow()[n as usize].get(word).cloned());
        if let Some(sum) = cached {
            return sum;
        }
    }

    let mut sum = 0;
    for (i, &c) in word.iter().enumerate() {
        if c == n - 1 {
            sum += rec_check_perm(n - 1, &word[(i + 1)..], cache_limit);
        }
    }
    if word.len() < cache_limit {
        SUBPERM_TABLES.with(|st| st.borrow_mut()[n as usize].insert(word.into(), sum));
    }
    sum
}

pub fn is_dice_word_fair(word: &[u8], cache_limit: usize) -> bool {
    let n = word.iter().max().unwrap() + 1;
    //let n_perms = (0 .. n as usize).product();
    let mut count = None;
    let perms = (0..n).permutations(n as usize).collect_vec();
    for p in perms {
        let w2 = word.iter().map(|&x| p[x as usize]).collect_vec();
        let c = rec_check_perm(n, &w2, cache_limit);
        if count.is_none() {
            count = Some(c);
        }
        if c != count.unwrap() {
            return false;
        }
    }
    true
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
        assert_eq!(mf.iterate_words().collect::<Vec<_>>(), &[Word::from_slice(&[2, 3, 0, 2, 2, 0, 3])]);
        assert_eq!(mf.iterate_words_subset(&[3]).collect::<Vec<_>>(), &[Word::from_slice(&[3, 3])]);
        assert_eq!(mf.iterate_words_subset(&[1]).collect::<Vec<_>>(), &[Word::from_slice(&[])]);
        assert_eq!(
            mf.iterate_words_subset(&[0, 1, 2]).collect::<Vec<_>>(),
            &[Word::from_slice(&[2, 0, 2, 2, 0])]
        );

        let mf2 = f.mapped_as(&[0, -1, 1, 2]);
        assert_eq!(mf2.iterate_words().collect::<Vec<_>>(), &[Word::from_slice(&[2, 3, 0, 2, 2, 0, 3])]);
    }
}
