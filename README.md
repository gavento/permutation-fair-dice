# Permutation fair dice

An implementation of an algorithm to find all _permutation-fair dice_ given the dice sizes, by Tomáš Gavenčiak and Václav Rozhoň.

---

Picture a group of friends engaging in a game where they should play in a random order. They may
each just roll a die and line up according to the numbers rolled, though this may require re-rolls on ties. A
natural question arises: Is it possible to design a set of dice with unique numbers on each side, ensuring
that there are no ties and yet the permutation in which the n friends line up is uniformly random? Such
a set of dice is referred to as permutation-fair dice.

In our algorithm, the dice are encoded as a sequence of letters, where _i_-th letter indicates which dice contains number _i_.
E.g. `ABCCBABACCABCBAABC` encodes dice A with faces 1, 6, 8, 11, 15, 16, dice B with faces 2, 5, 7, 12, 14, 17, and dice C with faces 3, 4, 9, 10, 13, 18.
This particular (6, 6, 6)-faced set of dice is also permutation-fair.

## Running the Rust version

Install the [Rust compiler toolchain](https://rustup.rs/), then check out this repository, compile in release mode, and run with desired dice sizes.

```
git clone https://github.com/gavento/permutation-fair-dice
cd permutation-fair-dice

# Build with rust cargo
cargo build --release

# Run with desired dice sizes
./target/release/main 6 6 6

# Note you can also look for dice fair only w.r.t the distribution of the first k players (rather than all players)
./target/release/main 4 6 6 6 --fair-up-to 3
```

Example output:

```
[00:00:00.000] INFO   # Gathering data for FDTS [6,6,6] (fair up to 3) ...
[00:00:00.000] INFO   # Gathering data for FDTS [6,6] (fair up to 2) ...
[00:00:00.000] INFO   Combining ([_,6]) and ([6,_]) into ([6,6]), checking []
combining: 100%|##########################################################################################################| 1/1 pairs [0s<0s] 29 results, 462 candidates, 119785.41 cands/s
[00:00:00.006] INFO   # Saved FDTS [[6,6]] (fair up to 2, 29 dice tuples) to "fdts_data/fdts_6_6_fair2.json"
[00:00:00.008] INFO   # Read FDTS [6,6] (fair up to 2, 29 dice tuples) from "fdts_data/fdts_6_6_fair2.json"
[00:00:00.009] INFO   # Read FDTS [6,6] (fair up to 2, 29 dice tuples) from "fdts_data/fdts_6_6_fair2.json"
[00:00:00.009] INFO   Combining ([6,_,6]) and ([6,6,_]) into ([6,6,6]), checking ["[_,6,6]"]
combining: 100%|######################################################################################################| 841/841 pairs [0s<0s] 11 results, 2421 candidates, 46386.31 cands/s
[00:00:00.062] INFO   # Saved FDTS [[6,6,6]] (fair up to 3, 11 dice tuples) to "fdts_data/fdts_6_6_6_fair3.json"
```

The last line indicates how many dice tuples exist (11 here, up to relabelling of the dice) or 0 if no such dice exist. The JSON files then contain the lists of the dice. Note that this does not take left-right symmetry into account. On subsequent runs the results for already enerated dice are read from the cache.

### Performance

On my laptop (Thinkpad L390 with Intel i5), all 12 fair dice of sizes [6, 6, 12, 12] are found under 2 minutes. Note that most of the computation is usually spent on finding all the (numerous) fair dice for a subset of dice with unnecessarily many sides - here 80% of the time was spent generating all 44902 fair [6, 12, 12] dice.

### Sketch of the algorithm

To build up the list of all target fair dice, we first compute relevant smaller-size fair dice and then combine them into candidate target fair dice, and check for permutation fairness. 

For example, for any fair dice (labeled ABC) of sizes [3, 4, 6], any two dice of the three also have to be fair. So we first find all fair AB-dice of sizes [3, 4], all fair AC-dice of sizes [3, 6], and all BC-dice of sizes [4, 6], and then we combine all the valid ABC dice from AB and AC dice by taking all interleavings of the two, while limiting the search only to the cases consistent with the BC subdice being fair. 

## Python implementation

`fairdice.ipynb` contains an older implementation of the algorithm. With it, you need to manually specify the order of the relabel and combine operations, it is much slower and does no result caching.
