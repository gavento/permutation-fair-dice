# Permutation fair dice

An implementation of an algorithm to find all _permutation fair dice_ given the dice sizes, by Tomáš Gavenčiak and Václav Rozhoň.

_Definition._ Consider sets of dice where all the sides have different number, even across the dice. Then every throw
determines a permutation of the dice if we order them according to the number shown.
Those dice are _permutation fair_ if every permutation of the dice has the same chance of occuring after a throw.

_Note._ The dice are encoded as a sequence of letters, i-th letter indicates which dice contains number i.
E.g. `ABCCBABACCABCBAABC` encodes dice A with sides 1, 6, 8, 11, 15, 16, dice B with sides 2, 5, 7, 12, 14, 17, and dice C with sides 3, 4, 9, 10, 13, 18.

## Running the Rust version

Install the Rust compiler, then check out this repository, compile in release mode for speed, and run with desired dice sizes.

```
git clone https://github.com/gavento/permutation-fair-dice
cd permutation-fair-dice
# Build with rust cargo
cargo build --release
# Run with desired dice sizes
./target/release/main 6 6 6
```

This outputs:

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

The last line indicates how many dice tuples exist (11 here, up to relabelling of the dice) or 0 if no such dice exist. The JSON files then contain the lists of the dice. Note that this does not take left-right symmetry into account. Note that on subsequent runs the results for already enerated dice are read from the cache.

### Performance

On my laptop (Thinkpad L390 with Intel i5), all 12 fair dice of sizes [6, 6, 12, 12] are found under 2 minutes. Note that most of the computation is usually spent on finding all the fair dice of a smaller number of dice but with unnecessarily too many sides - here 80% of the time was spent generating all 44902 fair [6, 12, 12] dice.

### Sketch of the algorithm

To build up the list of all target fair dice, we first compute relevant smaller-size fair dice and then combine them into candidate target fair dice, and check for permutation fairness. 

For example, for any fair dice (labeled ABC) of sizes [3, 4, 6], any two dice of the three also have to be fair. So we first find all fair AB-dice of sizes [3, 4], all fair AC-dice of sizes [3, 6], and all BC-dice of sizes [4, 6], and then we combine all the valid ABC dice from AB and AC dice by taking all interleavings of the two, while limiting the search only to the cases consistent with the BC subdice being fair. 

## Python implementation

`fairdice.ipynb` contains an older implementation of the algorithm. With it, you need to manually specify the order of the relabel and combine operations, it is much slower and does no result caching.