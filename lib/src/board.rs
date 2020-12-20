use crate::grid::{
    Cell::{LetterBonus, WordBonus},
    Grid,
};
use crate::tilesets::{Language, TileSet};
use crate::wordlist::{LetterSet, RowData, Wordlist};
use crate::{Cell, Codec, Error, Item, ItemList, Letter, Letters, List, Row, Tile, Word};

#[cfg(feature = "flame_it")]
use flamer::flame;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use std::fmt;

const N: usize = 15;
type State = [Row; N];

type Score = (usize, usize, bool, Word, u32);

/// Display the board state as 15 lines of 15 squares.
/// Empty squares show as ".".
impl<'a> fmt::Display for Board<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let repr = self
            .horizontal
            .iter()
            .map(|&row| self.wordlist.decode(row).replace(" ", "."))
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", repr)
    }
}

impl<'a> Default for Board<'a> {
    fn default() -> Self {
        Self::new(Language::EN)
    }
}

/// Represents the state of a `wordfeud` board.
/// * A grid of 15x15 squares with possible letter/word bonus,
/// * The tile distribution for language used (number of letters, and value of each letter),
/// * The wordlist used for the game.
#[derive(Debug, Clone)]
pub struct Board<'a> {
    board: Grid,
    empty_row: Row,
    horizontal: State,
    vertical: State,
    rowdata: [[RowData; N]; 2],
    tileset: TileSet<'a>,
    wordlist: Wordlist,
}

impl<'a> Board<'a> {
    /// Create a new empty `wordfeud ` board, with 15x15 squares.
    /// The [`language`](crate::Language) is used to specify the tile distribution used in the game.
    /// See [Wordfeud Help](https://wordfeud.com/wf/help/): Tile Distribution.
    /// Currently supported:
    /// - `EN` (english),
    /// - `NL` (dutch),
    /// - `SE` (swedish)
    ///
    /// ## Examples
    ///
    /// Basic usage:
    ///```
    /// use wordfeud_solver::{Board, Language};
    ///
    /// let board = Board::new(Language::NL);
    ///```
    /// Additional builder functions can be used to set the wordlist, grid and state of the board.
    /// See also:
    /// - [`with_wordlist_from_file`](Board::with_wordlist_from_file)
    /// - [`with_wordlist_from_words`](Board::with_wordlist_from_words)
    /// - [`with_state_from_strings`](Board::with_state_from_strings)
    /// - [`with_grid_from_strings`](Board::with_grid_from_strings)
    #[must_use]
    pub fn new(language: Language) -> Board<'a> {
        let tileset = TileSet::new(language);
        // Creating an empty wordlist never fails, so it safe to unwrap
        let wordlist = Wordlist::from_words(&[], tileset.codec()).unwrap();
        let grid = Grid::default();
        let empty_row = wordlist.encode("               ").unwrap();
        let mut empty_rowdata = RowData::new();
        for _ in 0..N {
            empty_rowdata.push((LetterSet::new(), false));
        }

        Board {
            board: grid,
            empty_row,
            horizontal: [empty_row; N],
            vertical: [empty_row; N],
            rowdata: [[empty_rowdata; N], [empty_rowdata; N]],
            tileset,
            wordlist,
        }
    }

    /// Specify the wordlist by reading it from `wordfile`, and returns the modified board.
    ///
    /// The `wordfile` must contain one word per line, and the words should be from language
    /// specified in the board.
    ///
    /// ## Errors
    /// This function will give an error if the `wordfile` does not exist, or cannot be encoded.
    /// ## Examples
    /// ```
    /// # use wordfeud_solver::{Board, Error};
    /// let board = Board::default().with_wordlist_from_file("../wordlists/words.txt")?;
    /// # Ok::<(), Error>(())
    /// ```
    pub fn with_wordlist_from_file(mut self, wordfile: &str) -> Result<Board<'a>, Error> {
        self.wordlist = Wordlist::from_file(wordfile, self.codec())?;
        self.set_rowdata();
        Ok(self)
    }

    /// Specify the wordlist by a list of words, and returns the modified board.
    /// ## Errors
    /// If the words can not be encoded.
    /// ## Example
    /// ```no_run
    /// use wordfeud_solver::Board;
    /// let board = Board::default().with_wordlist_from_words(&["aardvark", "zebra"]);
    ///```
    pub fn with_wordlist_from_words(mut self, words: &[&str]) -> Result<Board<'a>, Error> {
        self.wordlist = Wordlist::from_words(words, self.codec())?;
        self.set_rowdata();
        Ok(self)
    }

    #[cfg(feature = "bincode")]
    /// Specify the wordlist by deserializing it from the `wordfile`, and returns the modified board.
    ///
    /// The `wordfile` should be encoded with [`bincode`](https://docs.rs/bincode/1.3.1/bincode/) file
    ///
    /// ## Errors
    /// This function will give an error if the `wordfile` does not exist, or cannot be decoded.
    pub fn with_wordlist_deserialize_from(mut self, wordfile: &str) -> Result<Board<'a>, Error> {
        self.wordlist = Wordlist::deserialize_from(wordfile)?;
        self.set_rowdata();
        Ok(self)
    }

    /// Parse board state from a list of strings.
    /// The list must contain 15 rows of 15 characters.
    /// ## Errors
    /// If the list of strings has wrong dimensions or cannot be parsed as rows.
    ///
    /// ## Examples
    /// ```
    /// use wordfeud_solver::Board;
    /// let state = &[
    /// "    t     c   f",
    /// "    e    he   o",
    /// "    r   bis g k",
    /// "    u  bol te v",
    /// "    gepof dimme",
    /// "      la vree e",
    /// "    qua   ene  ",
    /// "      Spoelen  ",
    /// "     s a   n   ",
    /// "     c d we    ",
    /// "     hadden    ",
    /// "    nu o   y   ",
    /// "  wrat siJzen  ",
    /// "    k     os   ",
    /// "   zerk   g    ",
    /// ];
    /// let board = Board::default().with_state_from_strings(state);
    /// ```
    pub fn state_from_strings(&mut self, rows: &[&str]) -> Result<State, Error> {
        if rows.len() != N {
            return Err(Error::InvalidRowCount(rows.len()));
        }
        let mut state = [Row::new(); N];
        for (i, &row) in rows.iter().enumerate() {
            let encoded = self.wordlist.encode(row)?;
            if encoded.len() != N {
                return Err(Error::InvalidRowLength(String::from(row), encoded.len()));
            }
            state[i] = encoded;
        }
        Ok(state)
    }

    /// Set board state from list of strings
    pub fn set_state_from_strings(&mut self, rows: &[&str]) -> Result<(), Error> {
        let state = self.state_from_strings(rows)?;
        self.set_state(&state);
        Ok(())
    }

    /// Set board state from list of strings
    pub fn with_state_from_strings(mut self, rows: &[&str]) -> Result<Board<'a>, Error> {
        self.set_state_from_strings(rows)?;
        Ok(self)
    }

    /// Set board state from a list of rows, update rowdata
    pub fn set_state(&mut self, rows: &State) {
        self.horizontal = *rows;
        for i in 0..N {
            for j in 0..N {
                self.vertical[j][i] = self.horizontal[i][j];
            }
        }
        self.set_rowdata();
    }

    /// Set board cells from string representation
    /// ## Errors
    /// If the grid has wrong dimensions or cannot be parsed as valid board cells.
    pub fn set_grid_from_strings<S: AsRef<str>>(&mut self, grid: &[S]) -> Result<(), Error> {
        self.board = Grid::from_strings(grid)?;
        Ok(())
    }

    /// Set board cells from string representation
    /// ## Errors
    /// If the grid has wrong dimensions or cannot be parsed as valid board cells.
    pub fn with_grid_from_strings<S: AsRef<str>>(mut self, grid: &[S]) -> Result<Board<'a>, Error> {
        self.set_grid_from_strings(grid)?;
        Ok(self)
    }

    /// Return reference to our wordlist
    pub fn wordlist(&self) -> &Wordlist {
        &self.wordlist
    }

    /// Return the board horizontal state
    pub fn horizontal(&self) -> State {
        self.horizontal
    }

    /// Return the board vertical state
    pub fn vertical(&self) -> State {
        self.vertical
    }

    /// Return the grid
    pub fn grid(&self) -> Grid {
        self.board.clone()
    }

    /// Return tileset
    pub fn tileset(&self) -> &'a TileSet {
        &self.tileset
    }

    /// Check if cell at x, y is occupied.
    ///
    /// ## Examples
    /// ```
    /// # use std::convert::TryFrom;
    /// # use wordfeud_solver::{Board, Word, Error};
    /// let mut board = Board::default();
    /// board.play_word("aardvark", 7, 7, true, true)?;
    /// assert!(board.is_occupied(7,7));
    /// # Ok::<(), Error>(())
    pub fn is_occupied(&self, x: usize, y: usize) -> bool {
        if (x < N) && (y < N) {
            self.vertical[x][y].tile().is_some()
        } else {
            false
        }
    }

    fn calc_rowdata(&self, horizontal: bool, i: usize) -> RowData {
        let sw = self.surrounding_words(horizontal, i);
        let labelsets = sw
            .iter()
            .map(|surrounding| self.wordlist.get_legal_characters(surrounding))
            .collect::<Vec<_>>();
        let mut connected = sw
            .iter()
            .map(|surrounding| !surrounding.is_empty_cell())
            .collect::<Vec<_>>();
        if i == 7 {
            connected[7] = true;
        }
        let rowdata: RowData = labelsets
            .iter()
            .zip(connected)
            .map(|(&l, c)| (l, c))
            .collect();
        rowdata
    }

    fn set_rowdata(&mut self) {
        for i in 0..N {
            self.rowdata[0][i] = self.calc_rowdata(false, i);
            self.rowdata[1][i] = self.calc_rowdata(true, i);
        }
    }

    /// Play word at x, y on the board in given direction.
    /// Modifies the board state if `modify` is true.
    /// Returns the used letters, in the order of use.
    /// Letters in `word` that are already on the board are not included.
    /// <br>**NOTE**: if a letter in `word` is placed on the board in a position where
    /// a different letter is already used, the letter on the board is silently overwritten.
    /// ## Errors
    /// - If `word` cannot be encoded to valid [`Word`](crate::Word]).
    /// - If the placed `word` does not on the board.
    /// ## Examples
    /// ```
    /// # use std::convert::TryFrom;
    /// # use wordfeud_solver::{Board, Word, Error};
    /// let mut board = Board::default();
    /// let word = "aardvark";
    /// let used = board.play_word(word, 7,7,true, true)?;
    /// assert_eq!(used, word);
    ///# Ok::<(), Error>(())
    /// ```
    pub fn play_word(
        &mut self,
        word: &str,
        x: usize,
        y: usize,
        horizontal: bool,
        modify: bool,
    ) -> Result<String, Error> {
        let word = self.encode(word)?;
        let used_letters = self.try_word(word, x, y, horizontal)?;
        if modify {
            self.play_word_unchecked(word, x, y, horizontal);
        }
        Ok(self.decode(used_letters))
    }

    pub fn play_word_unchecked(&mut self, word: Word, x: usize, y: usize, horizontal: bool) {
        let mut x = x;
        let mut y = y;
        let (dx, dy) = if horizontal { (1, 0) } else { (0, 1) };
        for tile in word {
            self.horizontal[y][x] = tile.into_cell();
            x += dx;
            y += dy;
        }
        self.set_state(&self.horizontal.clone());
    }

    fn try_word(&self, word: Word, x: usize, y: usize, horizontal: bool) -> Result<Letters, Error> {
        let mut x = x;
        let mut y = y;
        let len = word.len();
        let (dx, dy) = if horizontal { (1, 0) } else { (0, 1) };
        if (x + len * dx > N) || (y + len * dy > N) {
            return Err(Error::TilePlacementError {
                x,
                y,
                horizontal,
                len,
            });
        }
        let mut used_letters = Letters::new();
        for tile in word {
            let c = self.horizontal[y][x];
            if c.tile().is_none() {
                used_letters.push(Letter::from_tile(tile));
            } else if c.tile() != Some(tile) {
                return Err(Error::TileReplaceError { x, y });
            }
            x += dx;
            y += dy;
        }
        Ok(used_letters)
    }

    /// Returns the the surrounding characters that would need to form a valid
    /// word in order to fill each position in the i'the row of the board
    #[cfg_attr(feature = "flame_it", flame)]
    fn surrounding_words(&self, horizontal: bool, i: usize) -> Vec<Row> {
        let mut res = Vec::new();
        let crossing_rows = if horizontal {
            self.vertical
        } else {
            self.horizontal
        };
        for row in &crossing_rows {
            res.push(row.surrounding_word(i));
        }
        res
    }

    /// Calculates the score of `word` placed at `x0`, `y0`, `horizontal` that has
    /// not yet been played on the board.
    /// If `include_crossing_words` is `true` the points for words that are created or extended
    /// in the crossing direction are included in the points. `include_crossing_words` should be
    /// set to `true` for normal use. It is set to `false` when `calc_word_points` calls itself
    /// recursively.
    /// ## Errors
    /// - If the placed `word` would cross the right or bottom boarder.
    /// ## Examples
    /// ```
    /// # use std::convert::TryFrom;
    /// # use wordfeud_solver::{Board, Word, Error};
    /// let board = Board::default();
    /// let word = board.encode("wordfeud")?;
    /// let points = board.calc_word_points(&word, 7, 7, true, true)?;
    /// assert_eq!(points, 78);
    /// # Ok::<(), Error>(())
    /// ```
    /// In this example, the values of the letters are: `w`:4 `o`:1, `r`:1, `d`:2, `f`:4, `u`:2.
    /// The `f` is on 2x word bonus, and the last `d` is on 2x letter bonus. The total value of the
    /// word is `2 x (4 + 1 + 1 + 2 + 4 + 1 +2 + (2 x 2)) = 2 x 19 = 38`. Because all 7 letters are played we get an extra
    /// "bingo" bonus of 40 points.  
    pub fn calc_word_points(
        &self,
        word: &Word,
        x: usize,
        y: usize,
        horizontal: bool,
        include_crossing_words: bool,
    ) -> Result<u32, Error> {
        let (dx, dy) = if horizontal { (1, 0) } else { (0, 1) };
        let len = word.len();
        if (x + len * dx > N) || (y + len * dy > N) {
            return Err(Error::TilePlacementError {
                x,
                y,
                horizontal,
                len,
            });
        }
        Ok(self.calc_word_points_unchecked(word, x, y, horizontal, include_crossing_words))
    }

    fn calc_word_points_unchecked(
        &self,
        word: &Word,
        x0: usize,
        y0: usize,
        horizontal: bool,
        include_crossing_words: bool,
    ) -> u32 {
        let mut word_multiplicator = 1;
        let mut word_points = 0;
        let mut tiles_used = 0;
        let mut total_points = 0;
        let (mut x, mut y) = (x0, y0);
        let (dx, dy) = if horizontal { (1, 0) } else { (0, 1) };

        for tile in word.into_iter() {
            let letter = tile.code();
            let mut letter_points = self.tileset.points(letter);
            if self.horizontal[y][x].tile().is_none() {
                tiles_used += 1;
                let square_bonus = self.board[y][x];
                match square_bonus {
                    LetterBonus(n) => {
                        letter_points *= n;
                    }
                    WordBonus(n) => {
                        word_multiplicator *= n;
                    }
                    _ => {}
                }
                if include_crossing_words {
                    let (crow, ci) = if horizontal {
                        (self.vertical[x], y)
                    } else {
                        (self.horizontal[y], x)
                    };
                    let row = crow;
                    let (s, e) = row.start_end(ci);
                    if e - s > 1 {
                        let (cx, cy) = if horizontal { (x, s) } else { (s, y) };
                        let cword =
                            Word::from(&row.replace(s, e, Cell::EMPTY, Cell::from_tile(tile)));
                        total_points +=
                            self.calc_word_points_unchecked(&cword, cx, cy, !horizontal, false);
                    }
                }
            }
            word_points += letter_points;
            x += dx;
            y += dy;
        }
        total_points += word_points * word_multiplicator;
        if tiles_used >= 7 {
            total_points += 40; // Bingo bonus
        }
        total_points
    }

    /// Returns a list with (`pos`, `word`) tuples for all words that can be played on `row`
    /// with index `i`, in direction `horizontal`, given `letters`.
    /// In the returned tuples, `pos` is the start index of the `word` in `row`.
    /// ## Examples
    /// ```
    /// # use std::convert::TryFrom;
    /// use wordfeud_solver::{Board, Letters, Row};
    /// let board = Board::default().with_wordlist_from_words(&["the", "quick", "brown", "fox"]).unwrap();
    /// let row = board.encode("               ").unwrap();
    /// let letters = board.encode("befnrowx").unwrap();
    /// let res = board.words(&row, true, 7, letters);
    /// assert_eq!(res.len(),8);
    /// ```
    /// In this example, we start with an empty board. The first word must have one of its letters
    /// on the centre square (7,7). The word "brown" can be placed at 3,4,5,6,7, and the word "fox" can be
    /// played at 5,6,7.
    pub fn words(
        &self,
        row: &Row,
        horizontal: bool,
        i: usize,
        letters: Letters,
    ) -> Vec<(usize, Word)> {
        let rowdata = self.rowdata[horizontal as usize][i];
        self.wordlist
            .words(&row, &rowdata, &letters, None)
            .collect()
    }

    /// Calculate the score for each word that can be played on the board with `letters`.
    /// Return a list of (`x`, `y`, `horizontal`, `word`, `score`) tuples.
    /// ## Examples
    /// ```
    /// # use wordfeud_solver::{Board, Error};
    /// let board = Board::default().with_wordlist_from_words(&["the", "quick", "brown", "fox"])?;
    /// let res = board.calc_all_word_scores("befnrowx")?;
    /// assert_eq!(res.len(),16);
    /// # Ok::<(), Error>(())
    /// ```
    /// In this example 16 results are returned: 8 in horizontal and 8 in vertical direction.
    /// See also [`Board::words`](Board::words).
    #[cfg_attr(feature = "flame_it", flame)]
    pub fn calc_all_word_scores(&self, letters: &str) -> Result<Vec<Score>, Error> {
        let letters = self.encode(letters)?;
        let mut scores: Vec<Score> = Vec::new();
        let hor_scores = |(i, row)| {
            let words = self.words(row, true, i, letters);
            let mut scores: Vec<Score> = Vec::new();
            for (x, word) in words {
                let points = self.calc_word_points_unchecked(&word, x, i, true, true);
                scores.push((x, i, true, word, points));
            }
            scores
        };
        let ver_scores = |(i, row)| {
            let words = self.words(row, false, i, letters);
            let mut scores: Vec<Score> = Vec::new();
            for (y, word) in words {
                let points = self.calc_word_points_unchecked(&word, i, y, false, true);
                scores.push((i, y, false, word, points));
            }
            scores
        };
        // #[cfg(feature = "rayon")]
        #[cfg(feature = "calc_all_word_scores_parallel")]
        {
            scores.par_extend(
                self.horizontal
                    .into_par_iter()
                    .enumerate()
                    .map(hor_scores)
                    .flatten(),
            );
            scores.par_extend(
                self.vertical
                    .into_par_iter()
                    .enumerate()
                    .map(ver_scores)
                    .flatten(),
            );
        }
        // #[cfg(not(feature = "rayon"))]
        #[cfg(not(feature = "calc_all_word_scores_parallel"))]
        {
            scores.extend(self.horizontal.iter().enumerate().map(hor_scores).flatten());
            scores.extend(self.vertical.iter().enumerate().map(ver_scores).flatten());
        }
        Ok(scores)
    }

    /// Return tile at x,y or None if empty cell  or outside grid.
    pub fn tile_at(&self, y: usize, x: usize) -> Option<Tile> {
        if x < N && y < N {
            return self.horizontal[y][x].tile();
        }
        None
    }

    fn codec(&self) -> &Codec {
        &self.tileset.codec()
    }

    /// Encode string to Word with our codec
    /// ## Errors
    /// If word can not be encoded
    pub fn encode<T: Item>(&self, word: &str) -> Result<ItemList<T>, Error> {
        self.wordlist.encode(word)
    }

    /// Decode word to string with our codec
    pub fn decode<T: Item>(&self, word: ItemList<T>) -> String {
        self.wordlist.decode(word)
    }

    // fn find_opponent_score(&self, our_tile_score: u32, letters: opp_words:Letters, in_endgame)

    /// Calculate the best opponent score with the given letters.
    ///
    /// The best opponent score is calculated by calling [calc_all_word_scores](Self::calc_all_word_scores) with the opponent `letters`.
    /// In endgame (0 letters left in bag) it also checks if the opponent can finish the game by playing all his remaining letters.
    /// In that case, `our_tile_score` is added to the best opponent score.
    /// If the opponent has to pass because there are no  possible moves, return 0 as score.
    /// # Errors
    /// Returns an error if [calc_all_word_scores](Self::calc_all_word_scores) fails.
    fn evaluate_opponent_scores(
        &self,
        letters: &str,
        our_tile_score: u32,
        in_endgame: bool,
    ) -> Result<(u32, bool), Error> {
        let mut result = self.calc_all_word_scores(letters)?;
        if !in_endgame {
            if result.is_empty() {
                // no possible moves for opponent, return 0
                return Ok((0, false));
            }
            result.sort_by(|a, b| (b.4).cmp(&a.4));
            return Ok((result[0].4, false));
        }
        let mut scores = vec![0];
        let mut exit_flag = false;
        for (x, y, horizontal, word, score) in result {
            let played = self.try_word(word, x, y, horizontal)?;
            let score = if played.len() == letters.len() {
                exit_flag = true;
                score + our_tile_score
            } else {
                score
            };
            scores.push(score);
        }
        scores.sort_by_key(|n| u32::MAX - n);

        Ok((scores[0], exit_flag))
    }

    /// calculate word scores for a list of racks
    #[cfg(not(feature = "rayon"))]
    pub fn sample_scores(
        &mut self,
        racks: &[String],
        our_tile_score: u32,
        in_endgame: bool,
    ) -> Result<Vec<(u32, bool)>, Error> {
        let opp_scores = racks
            .iter()
            .map(|letters| self.evaluate_opponent_scores(&letters, our_tile_score, in_endgame))
            .collect::<Result<Vec<_>, Error>>();
        opp_scores
    }

    #[cfg(feature = "rayon")]
    pub fn sample_scores(
        &self,
        racks: &[String],
        our_tile_score: u32,
        in_endgame: bool,
    ) -> Result<Vec<(u32, bool)>, Error> {
        let opp_scores = racks
            .par_iter()
            .map(|letters| self.evaluate_opponent_scores(&letters, our_tile_score, in_endgame))
            .collect::<Result<Vec<_>, Error>>();
        opp_scores
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    type Result<T> = std::result::Result<T, Error>;

    const TEST_STATE: &[&str] = &[
        "    t     c   f",
        "    e    he   o",
        "    r   bis g k",
        "    u  bol te v",
        "    gepof dimme",
        "      la vree e",
        "    qua   ene  ",
        "      Spoelen  ",
        "     s a   n   ",
        "     c d we    ",
        "     hadden    ",
        "    nu o   y   ",
        "  wrat siJzen  ",
        "    k     os   ",
        "   zerk   g    ",
    ];

    fn board_nl<'a>() -> Board<'a> {
        Board::new(Language::NL)
    }

    #[test]
    fn test_state() -> Result<()> {
        let mut board = board_nl().with_state_from_strings(&TEST_STATE)?;

        assert!(board.is_occupied(4, 0));
        assert!(!board.is_occupied(0, 0));
        assert!(board.is_occupied(14, 4));

        board.play_word("ster", 3, 0, true, true)?;
        assert_eq!(board.decode(board.horizontal[0]), "...ster...c...f");
        Ok(())
    }

    #[test]
    fn test_surrounding_words() -> Result<()> {
        let board = board_nl().with_state_from_strings(&TEST_STATE)?;
        let sw = board
            .surrounding_words(true, 8)
            .iter()
            .map(|&s| board.decode(s))
            .collect::<Vec<String>>();
        let expect = [
            ".", ".", ".", ".", ".", "schut", "plas.", "paddos", "o.", "e.we", "drel.en", "tienen",
            "gemeen.", ".", ".",
        ];
        println!("{:?}", expect);
        assert_eq!(sw, expect);
        Ok(())
    }

    #[test]
    fn test_calc_word_points() -> Result<()> {
        let board = board_nl().with_state_from_strings(&TEST_STATE)?;
        let word = board.encode("ster")?;
        let points = board.calc_word_points(&word, 3, 0, true, true)?;
        assert_eq!(7, points);
        let word = board.encode("abel")?;
        let points = board.calc_word_points(&word, 3, 6, false, true)?;
        assert_eq!(32, points);
        Ok(())
    }

    #[test]
    fn test_words() -> Result<()> {
        let words = &["af", "ja"];
        let board = board_nl()
            .with_wordlist_from_words(words)?
            .with_state_from_strings(&TEST_STATE)?;
        let letters: Letters = board.encode("j*")?;
        let i = 0;
        let horizontal = true;
        println!("{:?}", board.rowdata[1][0]);
        let words = board.words(&board.horizontal[0], horizontal, i, letters);
        assert_eq!(words.len(), 1);
        for (x, word) in words {
            println!("{} {} {}", x, i, board.decode(word)); // [(13, Word(['A', 'f']))]
        }
        Ok(())
    }

    #[test]
    fn test_calc_all_word_scores() -> Result<()> {
        let words = &[
            "af", "ah", "al", "aar", "aas", "be", "bi", "bo", "bar", "bes", "bel",
        ];
        let board = board_nl()
            .with_wordlist_from_words(words)?
            .with_state_from_strings(&TEST_STATE)?;

        let letters = "abel";
        let res = board.calc_all_word_scores(letters)?;
        // println!("{:?}", res);
        // board.verify_word_scores(&res);

        let expect = [
            (13, 0, true, "af", 5),
            (3, 1, true, "be", 5),
            (3, 1, true, "bel", 14),
            (13, 1, true, "bo", 9),
            (2, 2, true, "bar", 14),
            (3, 8, true, "bes", 8),
            (8, 6, false, "bo", 5),
        ];

        assert_eq!(expect.len(), res.len());
        for ((rx, ry, rhor, rword, rscore), (ex, ey, ehor, eword, escore)) in
            res.iter().zip(&expect)
        {
            assert_eq!(rx, ex);
            assert_eq!(ry, ey);
            assert_eq!(rhor, ehor);
            assert_eq!(&board.decode(*rword), eword);
            assert_eq!(rscore, escore);
        }
        Ok(())
    }

    #[test]
    fn test_board() {
        let board = board_nl().with_state_from_strings(&TEST_STATE).unwrap();
        println!("{}", board);
    }

    #[test]
    fn test_bingo() -> Result<()> {
        // playing all letters gets a 40 point bonus
        let board = board_nl();
        let word = board.encode("hoentje")?;
        let score = board.calc_word_points(&word, 7, 7, true, true)?;
        assert_eq!(score, 68);
        Ok(())
    }

    #[test]
    fn test_main() -> Result<()> {
        let mut board = board_nl().with_wordlist_from_words(&["rust", "rest"])?;
        let letters = "rusta";
        let results = board.calc_all_word_scores(letters)?;
        for (x, y, horizontal, word, score) in results {
            println!(
                "{} {} {} {} {}",
                x,
                y,
                horizontal,
                board.decode(word),
                score
            );
        }
        board.play_word("rust", 7, 7, true, true)?;
        println!("{}", board);
        Ok(())
    }

    #[test]
    #[should_panic(expected = "TileReplaceError { x: 7, y: 7 }")]
    fn test_tile_replace_error() {
        let mut board = Board::default();
        board.play_word("rust", 7, 7, true, true).unwrap();
        // expect TileReplaceError
        board.play_word("bar", 7, 6, false, true).unwrap();
    }

    #[test]
    #[should_panic(expected = "TilePlacementError { x: 12, y: 7, horizontal: true, len: 4 }")]
    fn test_tile_placement_error() {
        let mut board = Board::default();
        board.play_word("rust", 12, 7, true, true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_sample_scores() -> Result<()> {
        let mut board = board_nl()
            .with_wordlist_from_file("../wordlists/wordlist-nl.txt")?
            .with_state_from_strings(&TEST_STATE)?;
        // let letters = "ehkmopp";
        // board.play_word("hoppe", 7,7, true, true)?;
        let racks: Vec<String> = [
            "ddenuvw", "eeijkvy", "abeinuy", "ceeehtv", "*bjjoov", "bbeeotu", "eghknrt", "ddnosuw",
            "aaadeln", "abeinnz", "aceeiin", "deilnoz", "eeeimmo", "eefioou", "aefnnnt", "*ejlmrr",
            "addeent", "adgimno", "emprstt", "bceekpy",
        ]
        .iter()
        .map(|&s| String::from(s))
        .collect();

        let now = Instant::now();
        let opp_scores = board.sample_scores(&racks, 0, false)?;
        println!("took {:?}", now.elapsed());
        println!("{:?}", opp_scores);
        Ok(())
    }
}
