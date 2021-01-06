use super::{
    codes::{Code, LETTER_MASK},
    item::Item,
    tile::Tile,
};
use crate::error::Error;
use std::convert::TryFrom;

/// A cell on the board that is either empty or contains a [`Tile`](crate::Tile)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Cell(Option<Tile>);

impl Item for Cell {
    fn code(&self) -> Code {
        self.0.map_or(0, |tile| tile.code())
    }
}

impl Cell {
    /// An empty cell
    pub const EMPTY: Self = Self(None);

    fn new(code: Code) -> Cell {
        if code == 0 {
            Cell(None)
        } else {
            Cell(Some(Tile::new(code)))
        }
    }

    /// Create new `Cell` from `Tile`
    pub fn from_tile(tile: Tile) -> Cell {
        Cell::new(tile.code())
    }

    /// Get the contained tile or None
    pub fn tile(&self) -> Option<Tile> {
        self.0
    }

    /// Check if the cell is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    /// Remove wildcard flag from wrapped `Tile`.
    pub fn to_letter(self) -> Cell {
        Cell::new(self.code() & LETTER_MASK)
    }
}

impl TryFrom<Code> for Cell {
    type Error = Error;
    fn try_from(code: Code) -> Result<Self, Self::Error> {
        match code {
            0 | 1..=31 | 65..=95 => Ok(Self::new(code)),
            _ => Err(Self::Error::InvalidTileCode(code)),
        }
    }
}

impl Into<Code> for Cell {
    fn into(self) -> Code {
        self.code()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_letter() {
        let cell = Cell::new(65);
        assert_eq!(cell.code(), 65);
        assert_eq!(cell.to_letter().code(), 1);
    }
}
