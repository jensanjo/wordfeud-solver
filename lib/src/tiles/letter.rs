use super::codes::{Code, Label, BLANK, LETTER_MASK, UNINIT};
use super::{Item, Tile};
use crate::error::Error;
use std::convert::TryFrom;
use std::num::NonZeroU8;

/// A letter that can be used as [`Tile`](crate::Tile) on the board.
///
/// Either a regular letter or a `blank` ("*") that can be used as any letter.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Letter(pub(super) NonZeroU8);

impl Default for Letter {
    fn default() -> Self {
        Self::new(UNINIT)
    }
}

impl Item for Letter {
    fn code(&self) -> Code {
        self.0.get()
    }
}

impl Letter {
    fn new(code: Code) -> Letter {
        let label = NonZeroU8::new(code).expect("label can't be 0");
        Letter(label)
    }

    /// Create `Letter` from `Tile`
    pub fn from_tile(tile: Tile) -> Letter {
        Letter::new(tile.label())
    }

    /// Return new blank
    pub fn blank() -> Letter {
        Letter(NonZeroU8::new(BLANK).unwrap())
    }

    /// Check if letter is `blank`
    pub fn is_blank(&self) -> bool {
        self.code() == BLANK
    }

    /// Get label for letter.
    pub fn label(&self) -> Label {
        self.code() & LETTER_MASK
    }
}

impl TryFrom<Code> for Letter {
    type Error = Error;
    fn try_from(code: Code) -> Result<Self, Self::Error> {
        match code {
            1..=31 | BLANK => Ok(Self::new(code)),
            _ => Err(Self::Error::InvalidLetterCode(code)),
        }
    }
}

impl Into<Code> for Letter {
    fn into(self) -> Code {
        self.0.get()
    }
}
