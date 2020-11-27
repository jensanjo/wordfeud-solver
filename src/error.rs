use crate::codec::Label;
use thiserror::Error;

#[derive(Error, Debug)]
/// Errors that can be returned
pub enum Error {
    /// Error reading wordfile
    #[error("Wordfile \"{path}\" could not be read")]
    ReadError {
        path: String,
        source: std::io::Error,
    },

    /// Error deserializing bincoded wordfile
    #[cfg(feature = "bincode")]
    #[error("Wordfile {0} could not be deserialized")]
    WordfileDeserializeError(String),

    /// Error when encoding a word to tile labels
    #[error("Encoder: invalid token '{0}'")]
    EncodeError(String),

    /// Error when decoding an invalid label
    #[error("Decode: invalid code {0}")]
    DecodeError(Label),

    /// Error parsing board state or grid from strings
    #[error("Invalid number of rows {0} (expect 15)")]
    InvalidRowCount(usize),

    /// Parsing a row on the board needs 15 cells
    #[error("Invalid row length {0} (expect 15)")]
    InvalidRowLength(usize),

    /// Error parsing bonus cell
    #[error("Invalid grid bonus cell: \"{0}\"")]
    GridParseError(String),

    /// Attempt to place (part of) a word outside the board
    #[error("Playing {len} tiles at x={x}, y={y} does not fit")]
    TilePlacementError {
        x: usize,
        y: usize,
        horizontal: bool,
        len: usize,
    },

    /// Attempt to replace a tile already on the board
    #[error("Attempt to replace tile at x:{x}, y:{y}")]
    TileReplaceError { x: usize, y: usize },
}
