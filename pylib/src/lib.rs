use pyo3::create_exception;
use pyo3::{basic::PyObjectProtocol, exceptions::PyException, prelude::*, PyErr};
use std::convert::From;
use wordfeud_solver::Language;

create_exception!(pywordfeud_solver, WordfeudException, PyException);

/// Score as returned to python: x, y, horizontal, word, score

#[pyclass]
struct Score {
    #[pyo3(get)]
    x: usize,
    #[pyo3(get)]
    y: usize,
    #[pyo3(get)]
    horizontal: bool,
    #[pyo3(get)]
    word: String,
    #[pyo3(get)]
    score: i32,
}

#[pyclass]
struct BestScore {
    #[pyo3(get)]
    x: usize,
    #[pyo3(get)]
    y: usize,
    #[pyo3(get)]
    horizontal: bool,
    #[pyo3(get)]
    word: String,
    #[pyo3(get)]
    score: i32,
    #[pyo3(get)]
    adj_score: i32,
    #[pyo3(get)]
    played: String,
    #[pyo3(get)]
    exit_flag: u8,
    #[pyo3(get)]
    std: f32,
}

#[pyclass]
struct Board {
    _board: wordfeud_solver::Board<'static>,
}

#[pymethods]
impl Board {
    #[new]
    #[args(wordfile = "None", state = "None", grid = "None")]
    fn new(
        lang: &str,
        wordfile: Option<&str>,
        state: Option<Vec<&str>>,
        grid: Option<Vec<&str>>,
    ) -> PyResult<Self> {
        let language = match lang {
            "NL" => Ok(Language::NL),
            "EN" => Ok(Language::EN),
            "SE" => Ok(Language::SE),
            _ => Err(PyErr::new::<WordfeudException, String>(format!(
                "unsupported language: {}",
                lang
            ))),
        }?;
        let mut board = wordfeud_solver::Board::<'static>::new(language);
        if let Some(wordfile) = wordfile {
            board = board
                .with_wordlist_from_file(wordfile)
                .map_err(WordfeudError::from)?;
        }
        if let Some(state) = state {
            board = board
                .with_state_from_strings(&state)
                .map_err(WordfeudError::from)?;
        }
        if let Some(grid) = grid {
            board = board
                .with_grid_from_strings(&grid)
                .map_err(WordfeudError::from)?;
        }
        Ok(Board { _board: board })
    }

    fn set_state(&mut self, rows: Vec<&str>) -> PyResult<()> {
        let state = self
            ._board
            .state_from_strings(&rows)
            .map_err(WordfeudError::from)?;
        self._board.set_state(&state);
        Ok(())
    }

    fn set_grid(&mut self, grid: Vec<&str>) -> PyResult<()> {
        self._board
            .set_grid_from_strings(&grid)
            .map_err(WordfeudError::from)?;
        Ok(())
    }

    #[getter]
    fn get_board(&self) -> Vec<String> {
        self._board.grid().to_strings()
    }

    #[getter]
    fn get_horizontal(&self) -> Vec<String> {
        self._board
            .horizontal()
            .iter()
            .map(|&row| self._board.decode(row))
            .collect::<Vec<String>>()
    }

    #[getter]
    fn get_vertical(&self) -> Vec<String> {
        self._board
            .vertical()
            .iter()
            .map(|&row| self._board.decode(row))
            .collect::<Vec<String>>()
    }

    #[text_signature = "($self, letters: str)"]
    /// Calculate all words scores with given letters.
    /// Returns a list of (x,y,horizontal,word,score).
    fn calc_all_word_scores(&self, letters: String) -> PyResult<Vec<Score>> {
        let scores: Vec<_> = self
            ._board
            .calc_all_word_scores(&letters)
            .map_err(WordfeudError::from)?
            .into_iter()
            .map(|s| Score {
                x: s.x,
                y: s.y,
                horizontal: s.horizontal,
                word: self._board.decode(s.word),
                score: s.score as i32,
            })
            .collect();
        Ok(scores)
    }

    #[text_signature = "($self, letters, n)"]
    /// Calculate `n` best words scores with given letters.
    /// Returns a list of (x,y,horizontal,word,score).
    fn calc_top_scores(&self, letters: String, n: usize) -> PyResult<Vec<Score>> {
        let mut results: Vec<_> = self
            ._board
            .calc_all_word_scores(&letters)
            .map_err(WordfeudError::from)?;
        results.sort_by_key(|item| std::cmp::Reverse(item.score));
        Ok(results
            .into_iter()
            .take(n)
            .map(|s| Score {
                x: s.x,
                y: s.y,
                horizontal: s.horizontal,
                word: self._board.decode(s.word),
                score: s.score as i32,
            })
            .collect())
    }

    #[text_signature = "($self, word, x, y, horizontal, modify)"]
    #[args(modify = "true")]
    /// Play a word on the board at position x, y, direction.
    /// Returns the used letters. Modifies the board if modify is true
    fn play_word(
        &mut self,
        word: &str,
        x: usize,
        y: usize,
        horizontal: bool,
        modify: bool,
    ) -> PyResult<String> {
        let used_letters = self
            ._board
            .play_word(word, x, y, horizontal, modify)
            .map_err(WordfeudError::from)?;
        Ok(used_letters)
    }

    #[text_signature = "($self, racks, our_tile_score, in_endgame)"]
    fn sample_scores(
        &mut self,
        racks: Vec<&str>,
        our_tile_score: u32,
        in_endgame: bool,
    ) -> PyResult<Vec<(u32, bool)>> {
        let result = self
            ._board
            .sample_scores(&racks, our_tile_score, in_endgame)
            .map_err(WordfeudError::from)?;
        Ok(result)
    }

    fn find_best_score(&mut self, rack: &str, nsamples: usize) -> PyResult<Vec<BestScore>> {
        let rack = self._board.encode(rack).map_err(WordfeudError::from)?;
        let scores = wordfeud_solver::find_best_scores(&mut self._board, rack, nsamples)
            .map_err(WordfeudError::from)?;
        let results: Vec<BestScore> = scores
            .into_iter()
            .map(|s| BestScore {
                x: s.x,
                y: s.y,
                horizontal: s.horizontal,
                word: s.word,
                score: s.score,
                adj_score: s.adj_score,
                played: s.played,
                exit_flag: s.exit_flag as u8,
                std: s.std,
            })
            .collect();
        Ok(results)
    }
}

/// Wrapper around wordfeud_solver::Error so we convert to PyErr
struct WordfeudError(wordfeud_solver::Error);

impl From<wordfeud_solver::Error> for WordfeudError {
    fn from(err: wordfeud_solver::Error) -> WordfeudError {
        WordfeudError { 0: err }
    }
}

impl From<WordfeudError> for PyErr {
    fn from(err: WordfeudError) -> PyErr {
        PyErr::new::<WordfeudException, String>(err.0.to_string())
    }
}

#[pyproto]
impl PyObjectProtocol for Board {
    fn __repr__(&self) -> PyResult<String> {
        Ok(self._board.to_string())
    }
}

#[pyproto]
impl PyObjectProtocol for Score {
    fn __repr__(&self) -> String {
        let s = self;
        format!(
            "{{ x: {}, y: {}, horizontal: {}, word: {}, score: {} }}",
            s.x, s.y, s.horizontal, s.word, s.score
        )
    }
}

#[pyproto]
impl PyObjectProtocol for BestScore {
    fn __repr__(&self) -> String {
        let s = self;
        format!("{{ x: {}, y: {}, horizontal: {}, word: {}, score: {} adj_score: {} played: {} exit: {} std: {:.1} }}",
            s.x, s.y, s.horizontal, s.word, s.score, s.adj_score, s.played, s.exit_flag, s.std)
    }
}

#[pymodule]
fn pywordfeud_solver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Board>()?;
    Ok(())
}
