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

    /// The string is longer than 15 tokens
    #[error("Encoder: string too long {0}")]
    EncodeStringTooLong(String),

    /// Token can not be encoded
    #[error("Encoder: invalid token '{0}'")]
    EncodeInvalidToken(String),

    /// Code is not valid for `Tile` or `Cell`
    #[error("Invalid code for tile {0}")]
    InvalidTileCode(u8),

    /// Code is not valid for `Letter`
    #[error("Invalid code for letter {0}")]
    InvalidLetterCode(u8),

    /// Error parsing board state or grid from strings
    #[error("Invalid number of rows {0} (expect 15)")]
    InvalidRowCount(usize),

    /// Parsing a row on the board needs 15 cells
    #[error("Invalid row \"{0}\": length {1}, expect 15")]
    InvalidRowLength(String, usize),

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
