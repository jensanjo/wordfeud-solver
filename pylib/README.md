# The `pywordfeud` Python package

The `pywordfeud` package is a python wrapper for the Rust [wordfeud-solver](https://github.com/jensanjo/wordfeud-solver) crate.

## Installation

```
pip install pywordfeud
```

In addition to `pywordfeud` you will need a wordfeud wordlist for the language(s) you want to use.

## Usage

Here is an example:

```python
import pywordfeud
# create a board for dutch wordfeud with `wordfile`
board = pywordfeud.Board("NL", wordfile="wordlists/wordlist-nl.txt")
# put a first word on the board
board.play_word("hulpen", 7,7, True, True)
# calculate the 20 best scores with given letters
top20 = board.calc_top_scores("adunczh")
top20
```

Results in this output:
```[(10, 7, False, 'punch', 36),
 (9, 7, False, 'lach', 23),
 (12, 6, True, 'zn', 22),
 (10, 6, True, 'uh', 21),
 (6, 6, True, 'duh', 20),
 (6, 8, True, 'duh', 20),
 (9, 5, False, 'zul', 20),
 (9, 7, False, 'launch', 20),
 (11, 8, True, 'hun', 19),
 (8, 4, False, 'chaud', 19),
 (9, 5, False, 'zal', 19),
 (7, 6, True, 'uh', 18),
 (7, 8, True, 'uh', 18),
 (11, 8, True, 'hu', 18),
 (10, 7, False, 'pand', 18),
 (9, 6, True, 'au', 17),
 (10, 8, True, 'ah', 17),
 (11, 8, True, 'hand', 17),
 (9, 5, False, 'hul', 17),
 (9, 7, False, 'lunch', 17)]
```

```python
board.play_word("punch", 10,7, False, True) 
board
```

Output:
```
...............
...............
...............
...............
...............
...............
...............
.......hulpen..
..........u....
..........n....
..........c....
..........h....
...............
...............
...............
```