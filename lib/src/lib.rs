#![doc(
    html_logo_url = "https://github.com/jensanjo/wordfeud-solver/raw/master/images/logo.png",
    html_favicon_url = "https://github.com/jensanjo/wordfeud-solver/raw/master/images/logo.png"
)]
#![deny(clippy::wrong_pub_self_convention, clippy::used_underscore_binding,
//    clippy::pub_enum_variant_names,
//    clippy::missing_docs_in_private_items,
//    clippy::non_ascii_literal, clippy::unicode_not_nfc,
//    clippy::unwrap_used, 
   clippy::map_unwrap_or,
//    clippy::filter_map,
//    clippy::shadow_unrelated, clippy::shadow_reuse, clippy::shadow_same,
   clippy::int_plus_one, clippy::string_add_assign, clippy::if_not_else,
   clippy::invalid_upcast_comparisons,
//    clippy::cast_precision_loss, clippy::cast_lossless,
//    clippy::cast_possible_wrap, clippy::cast_possible_truncation,
   clippy::mutex_integer, clippy::mut_mut, clippy::items_after_statements,
   clippy::print_stdout, clippy::mem_forget, clippy::maybe_infinite_iter)]

//! A wordfeud library for Rust.
//! <br>
//! This crate allows you to calculate the best scores in a game of wordfeud.
//! It can be used to study strategies in the game, or just to cheat.
//! This library is a Rust port of the excellent [`wordfeudplayer`](https://github.com/mrcz/Wordfeud-Player) python library.
//! It can use the `rayon` crate to calculate moves in parallel.
//! The time required to evaluate a board is in the order of 1 millisecond.
//!
//! # How to use `wordfeud_solver`
//! Start by creating a wordfeud board, then specify the wordlist to be used, and the tiles on the board.
//! By default a standard board is used, but you can specify your own "random" board.
//! The wordlist must be in utf-8 and contain one word per line.
//! Several wordfeud wordlists are available on the internet.
//! A wordlist for the dutch language is available [here](https://github.com/jensanjo/wordfeud-wordlists).
//! It is based on the [`OpenTaal`](https://www.opentaal.org)
//! wordlist, with modifications by the author.
//!
//! # Basic usage
//!  ```
//! # use wordfeud_solver::{Board, Error};
//! let mut board = Board::default().with_wordlist_from_words(&["rust", "rest"])?;
//! let results = board.calc_all_word_scores("rusta")?;
//! assert_eq!(results.len(),8);
//! for s in results {
//!        println!("{} {} {} {} {}", s.x, s.y, s.horizontal, board.decode(s.word), s.score);
//!}
//! board.play_word("rust", 7, 7, true, true)?;
//! println!("{}", board);
//! # Ok::<(), Error>(())
//! ```
mod ai;
mod board;
mod error;
mod grid;
mod labelset;
mod tilebag;
mod tiles;
mod tilesets;
mod wordlist;

pub use crate::ai::{find_best_scores, remaining_tiles, Score as BestScore};
pub use crate::board::{Board, Score};
pub use crate::error::Error;
pub use crate::grid::Grid;
pub use crate::tiles::{
    Cell, Code, Codec, Item, ItemList, Label, Letter, Letters, List, Row, Tile, Word,
};
pub use crate::tilesets::Language;
pub use crate::tilesets::TileSet;
pub use crate::tilebag::TileBag;
pub use crate::wordlist::{RowData, Wordlist};
