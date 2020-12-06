/// Code 1..31 for valid letter in wordlist (a..z plus language specific)
pub type Label = u8;

/// Tile code used to represent `Tile` or `Letter`. See [`Codec`](crate::Codec).
pub type Code = u8;

/// code for EMPTY (no tile)
pub const EMPTY: Code = 0;

/// code for BLANK tile
pub const BLANK: Code = 0x40;

/// Mask to get label value 0..31
pub const LETTER_MASK: u8 = 0b11111;

/// bitflag for wildcard
pub const IS_WILDCARD: Code = 0x40;

/// An uninitialized tile
pub(super) const UNINIT: Code = 0x7f;
