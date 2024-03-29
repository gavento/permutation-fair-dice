use std::{sync::Mutex, time::Instant};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashMap as HashMap;

use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use log::{debug, info};

use crate::{is_word_permutation_fair_up_to, subset_word, DiceTuple, MappedFDTS, Word, FDTS};

impl FDTS {
    pub fn new_combined(d1: MappedFDTS<'_>, d2: MappedFDTS<'_>, checking: &[MappedFDTS<'_>], fair_up_to: usize) -> Self {
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
        f.fair_up_to = fair_up_to;
        assert!(f.fair_up_to <= f.n());
        assert!(f.fair_up_to >= d1.fdts.fair_up_to);
        assert!(f.fair_up_to >= d2.fdts.fair_up_to);
        for c in checking {
            assert!(f.fair_up_to >= c.fdts.fair_up_to);
        }

        let bin_indices: Vec<_> = d1.map.iter().cloned().filter(|i| d2.map.contains(i)).collect();
        info!(
            "Combining ({}) and ({}) into ({}), checking {:?}",
            d1.sizes_string(),
            d2.sizes_string(),
            f.sizes_string(),
            checking.iter().map(|c| { c.sizes_string() }).collect_vec()
        );

        if d1.fdts.dice.is_empty() || d2.fdts.dice.is_empty() || checking.iter().any(|c| c.fdts.dice.is_empty()) {
            debug!(" .. some of the inputs are empty, returning empty FDTS {}.", f.sizes_string());
            return f;
        }

        let mut bins1 = HashMap::<Word, Vec<Word>>::default();
        for w in d1.iterate_words() {
            bins1.entry(subset_word(&w, &bin_indices)).or_default().push(w);
        }

        let mut bins2 = HashMap::<Word, Vec<Word>>::default();
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

        debug!(
            " .. combining {} and {} dice with common positions {:?} and {} bins, interleaving total {} dice pairs, fair up to {}",
            d1.fdts.dice.len(),
            d2.fdts.dice.len(),
            &bin_indices,
            common_keys.len(),
            total_pairs,
            f.fair_up_to,
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
            let local_process = |w1, w2| {
                let mut local_c = 0;
                let mut local_res = Vec::new();
                for wi in f.interleave_words(w1, w2, &checking, &bin_indices, true) {
                    local_c += 1;
                    if is_word_permutation_fair_up_to(&wi, (0..f.n() as u8).collect::<Word>(), f.fair_up_to) {
                        local_res.push(DiceTuple::from_word(&f, &wi));
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
            };
            // If key_w1_pairs is small, also iterate over bins2[bw] in parallel
            if key_w1_pairs.len() > 64 {
                for w2 in &bins2[bw] {
                    local_process(w1, w2);
                }
            } else {
                bins2[bw].par_iter().for_each(|w2| local_process(w1, w2));
            }
        });
        bar.finish();
        for rd in res.into_inner().unwrap() {
            f.insert_dice_tuple(rd);
        }

        debug!(
            " .. created FDTS {:?} with {} fair DiceTuples ({} prefixes)",
            &f.sizes,
            f.dice.len(),
            f.prefixes.len(),
        );

        f
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
            let mut size_groups = HashMap::default();
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
}

#[cfg(test)]
mod test {
    use crate::FDTS;

    #[test]
    fn test_d6_d6() {
        let d6 = FDTS::new_single(6);
        assert_eq!(d6.dice.len(), 1);
        assert_eq!(d6.prefixes.len(), 7);
        let d6_d6 = FDTS::new_combined(d6.mapped_as(&[0, -1]), d6.mapped_as(&[-1, 0]), &[], 2);
        assert_eq!(d6_d6.dice.len(), 29);
        assert_eq!(d6_d6.prefixes.len(), 200);
    }
}
