//! Basic types for working with wordfeud tiles.
mod cell;
mod codec;
mod codes;
mod item;
mod items;
mod letter;
mod list;
mod tile;

/// Maximum length of Code list
pub(super) const DIM: usize = 16;
pub use cell::Cell;
pub use codec::Codec;
pub use codes::{Code, Label, BLANK};
pub use item::Item;
pub use items::{Letters, Row, TryIntoLetters, Word};
pub use letter::Letter;
pub use list::{ItemList, List};
pub use tile::Tile;
