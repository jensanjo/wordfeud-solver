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
//! A wordlist for the dutch language is available **TODO**. It is based on the [`OpenTaal`](https://www.opentaal.org)
//! wordlist, with modifications by the author.
//!
//! # Basic usage
//!  ```
//! # use std::convert::TryFrom;
//! use wordfeud_solver::{Board, Letters};
//!
//! let mut board = Board::default().with_wordlist_from_words(&["rust", "rest"])?;
//! let letters = Letters::try_from("rusta")?;
//! let results = board.calc_all_word_scores(letters);
//! assert_eq!(results.len(),8);
//! for (x,y,horizontal,word,score) in results {
//!        println!("{} {} {} {} {}", x, y, horizontal, word, score);
//!}
//! board.play_word("rust", 7, 7, true, true)?;
//! println!("{}", board);
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! # About implementation
//!
mod board;
mod codec;
mod error;
mod grid;
mod labelset;
mod tiles;
mod tilesets;
mod wordlist;

pub use board::Board;
pub use codec::Codec;
pub use error::Error;
pub use tiles::{Row, Tile, Tiles};
pub use tilesets::{Language, TileSet};
pub use wordlist::{Letters, RowData, Word, Wordlist};
