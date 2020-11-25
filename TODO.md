# TODO

- [x] Build board with wordlist, codec
- [ ] Fix `calc_all_word_scores_iter` with `rayon`
- [ ] Refactor `Tiles`, `Row`
    - [x] Remove `append`
- [ ] Documentation
- [ ] Improve codec performance
- [ ] Provide tilesets with codec, tile distribution
- [ ] Get everything on github!

## matches

signature:

```Rust
fn matches(
    &self,
    node: usize,
    row: &Row,
    rowdata: &RowData,
    pos: usize,
    letters: &Letters,
    word: Tiles,
    connecting: bool,
    extending: bool
) -> Vec<Word>
    
    let mut matches = Vec::new()
    if pos < row.len() {
        if there is a letter at row[pos] {
            if that letter is a child {
                extend matches recursive
            } 
        }
    }
```


## codesets

- code: 0..31     
- is_letter: 32
- is_wildcard: 64

codesets: 

- default codeset: 1..26 =>  A..Z

27 Ä
28 Ö
29 Ü
30
31

## codec

```Rust
fn encode(t: &str) -> Result<Tile,_>

fn decode(t: Tile) -> Result<Iterator<char>, _>
```

# Performance

Some ideas to improve performance:
* Cache rowdata in board. Then `calc_all_word_scores` can 
  be repeated with different racks without recalculating rowdata 
* Precalculate distance to next connected in row.
  In `wordlist.words`, if distance > number of remaining letters: skip to next connected
* An additional improvement can be made by calculating the 
  length of the longest possible prefix with given letters
* iterator over rowdata?
* matches return iterator?
* Rack: instead of removing letters from a vec: 
    - use a bitmap multiset, and decrement count for used letter, OR:
    - set a flag for used char in position, OR:
    - just zero the letter in rack, and skip it in subsequent tests

`wordlist.words` is called 2*15 times: once for each horizontal and vertical row.
`wordlist.matches` (top level) is called in the order of 2\*15\*15 times (typically ~ 300).


# Board

* simplify words: precalculate rowdata

