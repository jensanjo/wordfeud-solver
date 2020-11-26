use crate::codec::Label;
// use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
/// Error during encoding/decoding between tiles and strings.
pub enum Error {
    #[error("Wordfile {0} could not be read")]
    WordfileReadError(String),
    #[cfg(feature = "bincode")]
    #[error("Wordfile {0} could not be deserialized")]
    WordfileDeserializeError(String),
    #[error("Encoder: invalid token '{0}'")]
    EncodeError(String),
    #[error("Decode: invalid code {0}")]
    DecodeError(Label),
    #[error("Invalid number of rows {0} (expect 15)")]
    InvalidRowCount(usize),
    #[error("Invalid row length {0} (expect 15)")]
    InvalidRowLength(usize),
    #[error("Invalid grid bonus cell: \"{0}\"")]
    GridParseError(String),
    #[error("Board index out of bounds {x}, {y}, {horizontal}, {len}")]
    BoardIndexError {
        x: usize,
        y: usize,
        horizontal: bool,
        len: usize,
    },
}
