use pyo3::{basic::PyObjectProtocol, exceptions::PyException, prelude::*, PyErr};
use pyo3::create_exception;
use std::convert::From;
use wordfeud_solver::Language;

create_exception!(pywordfeud_solver, WordfeudException, PyException);

/// Score as returned to python
type Score = (usize, usize, bool, String, u32);

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
            .map(|(x, y, hor, word, score)| (x, y, hor, self._board.decode(word), score))
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
        results.sort_by(|a, b| (b.4).cmp(&a.4));    
        Ok(results.into_iter()
            .take(n)
            .map(|(x, y, hor, word, score)| (x, y, hor, self._board.decode(word), score))
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

#[pymodule]
fn pywordfeud_solver(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Board>()?;
    Ok(())
}
