use super::codes::{Code, Label, IS_WILDCARD, LETTER_MASK, UNINIT};
use super::{Cell, Item, Letter};
use crate::error::Error;
use std::convert::TryFrom;
use std::num::NonZeroU8;

/// A tile on the board, either a regular letter or a wildcard (blank used as letter)
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Tile(pub(super) NonZeroU8);

impl Default for Tile {
    fn default() -> Self {
        Self::new(UNINIT)
    }
}

impl Item for Tile {
    fn code(&self) -> Code {
        self.0.get()
    }
}

impl Tile {
    pub(super) fn new(code: Code) -> Tile {
        let label = NonZeroU8::new(code).expect("label can't be 0");
        Tile(label)
    }

    /// Create `Tile` from `Letter`
    pub fn from_letter(letter: Letter) -> Tile {
        Tile(letter.0)
    }

    /// Create `Cell` from tile
    pub fn into_cell(self) -> Cell {
        Cell::from_tile(self)
    }

    /// Return a wildcard tile for letter `code`.
    /// ## Example
    /// ```
    /// use wordfeud_solver::{Item,Tile};
    /// let tile = Tile::wildcard_from_letter(1);
    /// assert_eq!(tile.code(), 65);
    /// ```
    pub fn wildcard_from_letter(code: u8) -> Tile {
        Tile::new((code & LETTER_MASK) | IS_WILDCARD)
    }

    /// Check if the tile is a wildcard
    pub fn is_wildcard(&self) -> bool {
        self.code() & IS_WILDCARD != 0
    }

    /// Get label for tile, ignoring the wildcard attribute.
    /// ## Example
    /// ```
    /// use wordfeud_solver::{Item, Tile};
    /// let tile = Tile::wildcard_from_letter(1);
    /// assert_eq!(tile.code(), 65);
    /// assert_eq!(tile.label(), 1);
    /// ```
    pub fn label(&self) -> Label {
        self.code() & LETTER_MASK
    }
}

impl TryFrom<Code> for Tile {
    type Error = Error;
    fn try_from(code: Code) -> Result<Self, Self::Error> {
        match code {
            1..=31 | 65..=95 => Ok(Self::new(code)),
            _ => Err(Self::Error::InvalidTileCode(code)),
        }
    }
}

impl Into<Code> for Tile {
    fn into(self) -> Code {
        self.0.get()
    }
}
