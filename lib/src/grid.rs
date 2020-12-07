use crate::Error;
use std::fmt;
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

#[derive(Debug, Copy, Clone)]
pub enum Cell {
    NoBonus,
    Start,
    LetterBonus(u32),
    WordBonus(u32),
}

pub type Grid = [[Cell; N]; N];

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

/// Create a symmetrical `wordfeud` board by mirroring a quarter board
/// horizontally and vertically
pub fn expand_quarter_board(qb: &[&str; Q]) -> Grid {
    let mut board: Grid = [[NoBonus; N]; N];
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

pub fn default() -> Grid {
    expand_quarter_board(&_DEFAULT_QUARTER_BOARD)
}

/// Get string representation of board cells
#[allow(dead_code)]
pub fn as_array(grid: &Grid) -> Vec<Vec<String>> {
    grid.iter()
        .map(|row| row.iter().map(Cell::to_string).collect::<Vec<String>>())
        .collect::<Vec<_>>()
}

pub fn from_array(grid: &[Vec<String>]) -> Result<Grid, Error> {
    assert_eq!(grid.len(), N);
    if grid.len() != N {
        return Err(Error::InvalidRowCount(grid.len()));
    }
    let mut board: Grid = [[NoBonus; N]; N];

    for (i, row) in grid.iter().enumerate() {
        if row.len() != N {
            return Err(Error::InvalidRowLength(row.len()));
        }
        for (j, cell) in row.iter().enumerate() {
            let val = cell.parse().unwrap();
            board[i][j] = val;
        }
    }
    Ok(board)
}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    fn test_grid_from_array() -> Result<(), Error> {
        let grid = default();
        let grid_as_strings = as_array(&grid);
        println!("{:?}", grid_as_strings);
        assert_eq!(as_array(&from_array(&grid_as_strings)?), grid_as_strings);
        Ok(())
    }
}
