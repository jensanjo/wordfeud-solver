use crate::Error;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

const N: usize = 15;
const Q: usize = 1 + N / 2;

const _DEFAULT_QUARTER_BOARD: [&str; Q] = [
    "3l -- -- -- 3w -- -- 2l",
    "-- 2l -- -- -- 3l -- --",
    "-- -- 2w -- -- -- 2l --",
    "-- -- -- 3l -- -- -- 2w",
    "3w -- -- -- 2w -- 2l --",
    "-- 3l -- -- -- 3l -- --",
    "-- -- 2l -- 2l -- -- --",
    "2l -- -- 2w -- -- -- ss",
];

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Cell {
    NoBonus,
    Start,
    LetterBonus(u32),
    WordBonus(u32),
}

type Inner = [[Cell; N]; N];
/// Wordfeud board grid, consisting of 15x15 (normal or bonus) squares.
///
/// A bonus square has a 2x or 3x letter bonus, or a 2x or 3x word bonus.
/// The center square at (7,7) is the "start" square, and must be used in the first turn.
#[derive(Debug, Clone, PartialEq)]
pub struct Grid(Inner);

impl Deref for Grid {
    type Target = Inner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Grid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_strings().join("\n"))
    }
}

use Cell::{LetterBonus, NoBonus, Start, WordBonus};

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NoBonus => write!(f, "--"),
            Start => write!(f, "ss"),
            LetterBonus(n) => write!(f, "{}l", n),
            WordBonus(n) => write!(f, "{}w", n),
        }
    }
}

impl FromStr for Cell {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "--" => Ok(NoBonus),
            "ss" => Ok(Start),
            "2l" => Ok(LetterBonus(2)),
            "3l" => Ok(LetterBonus(3)),
            "2w" => Ok(WordBonus(2)),
            "3w" => Ok(WordBonus(3)),
            _ => Err(Error::GridParseError(String::from(s))),
        }
    }
}

impl Grid {
    /// Create a new empty grid 15x15 cells with no bonus.
    fn empty() -> Grid {
        Grid([[NoBonus; N]; N])
    }

    /// Create a symmetrical `wordfeud` board by mirroring a quarter board
    /// horizontally and vertically
    fn expand_quarter_board(qb: &[&str; Q]) -> Grid {
        let mut board = Grid::empty();
        for (i, row) in qb.iter().enumerate() {
            let row = row.split(' ').collect::<Vec<&str>>();
            assert!(row.len() == Q);
            for (j, c) in row.iter().enumerate() {
                let val = c.parse().unwrap();
                board[i][j] = val;
                board[N - i - 1][j] = val;
                board[i][N - j - 1] = val;
                board[N - i - 1][N - j - 1] = val;
            }
        }
        board
    }

    /// Create default wordfeud grid
    /// ## Example
    /// ```
    /// # use wordfeud_solver::Grid;
    /// let grid = Grid::default();
    /// println!("{}", grid);
    /// ```
    pub fn default() -> Grid {
        Grid::expand_quarter_board(&_DEFAULT_QUARTER_BOARD)
    }

    /// Get board cells as a vec of 15 strings
    pub fn to_strings(&self) -> Vec<String> {
        self.iter()
            .map(|row| {
                row.iter()
                    .map(Cell::to_string)
                    .collect::<Vec<String>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
    }

    /// Create a `Grid` from strings
    /// Parameter `grid` must have 15 rows, each row consisting of 15 elements joined by spaces.
    ///
    /// ## Errors
    /// If `grid` has wrong dimensions, or elements can not be parsed as a `Cell`.
    /// ## Examples
    /// ```
    /// # use wordfeud_solver::{Grid, Error};
    /// let grid_strings = &[
    /// "3l -- -- -- 3w -- -- 2l -- -- 3w -- -- -- 3l",
    /// "-- 2l -- -- -- 3l -- -- -- 3l -- -- -- 2l --",
    /// "-- -- 2w -- -- -- 2l -- 2l -- -- -- 2w -- --",
    /// "-- -- -- 3l -- -- -- 2w -- -- -- 3l -- -- --",
    /// "3w -- -- -- 2w -- 2l -- 2l -- 2w -- -- -- 3w",
    /// "-- 3l -- -- -- 3l -- -- -- 3l -- -- -- 3l --",
    /// "-- -- 2l -- 2l -- -- -- -- -- 2l -- 2l -- --",
    /// "2l -- -- 2w -- -- -- ss -- -- -- 2w -- -- 2l",
    /// "-- -- 2l -- 2l -- -- -- -- -- 2l -- 2l -- --",
    /// "-- 3l -- -- -- 3l -- -- -- 3l -- -- -- 3l --",
    /// "3w -- -- -- 2w -- 2l -- 2l -- 2w -- -- -- 3w",
    /// "-- -- -- 3l -- -- -- 2w -- -- -- 3l -- -- --",
    /// "-- -- 2w -- -- -- 2l -- 2l -- -- -- 2w -- --",
    /// "-- 2l -- -- -- 3l -- -- -- 3l -- -- -- 2l --",
    /// "3l -- -- -- 3w -- -- 2l -- -- 3w -- -- -- 3l",   
    /// ];
    /// let grid =  Grid::from_strings(grid_strings)?;
    /// assert_eq!(grid.len(), 15);
    /// assert_eq!(grid[0].len(), 15);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn from_strings<S: AsRef<str>>(grid: &[S]) -> Result<Grid, Error> {
        if grid.len() != N {
            return Err(Error::InvalidRowCount(grid.len()));
        }
        let mut board = Grid::empty();
        for (i, row) in grid.iter().enumerate() {
            let row: Vec<&str> = row.as_ref().split(' ').collect();
            if row.len() != N {
                return Err(Error::InvalidRowLength(row.len()));
            }
            for (j, &cell) in row.iter().enumerate() {
                let val = cell.parse()?;
                board[i][j] = val;
            }
        }
        Ok(board)
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    fn test_grid_from_array() -> Result<(), Error> {
        let grid = Grid::default();
        let grid_as_strings = grid.to_strings();
        println!("{:#?}", grid.to_strings());
        assert_eq!(Grid::from_strings(&grid_as_strings)?, grid);
        Ok(())
    }
}
