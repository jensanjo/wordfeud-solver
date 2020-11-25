use anyhow::Result;
use lazy_static::lazy_static;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::Iterator;
use std::str;
use thiserror::Error;

/// A label 1..31
pub type Label = u8;
/// Flag marking blank tile (wildcard)
pub const IS_WILDCARD: Label = 0x40;
/// An empty square
pub const EMPTY: Label = 0;
/// An unassigned blank tile
pub const BLANK: Label = IS_WILDCARD;
/// Mask to extract letter code (lower 5 bits)
pub const LETTER_MASK: Label = 0b11111;

const ASCII_LC: &str = "abcdefghijklmnopqrstuvwxyz";

lazy_static! {
    static ref DEFAULT_CODEC: Codec = Codec::new();
}

#[derive(Error, Debug)]
/// Error during encoding/decoding between tiles and strings.
pub enum CodecError {
    #[error("encoder: invalid token '{0}'")]
    EncodeError(String),
    #[error("decode: invalid code {0}")]
    DecodeError(Label),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct CodeSet {
    encoder: HashMap<String, Label>,
    decoder: HashMap<Label, String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Translate from string to label codes and vice versa.
/// Each wordfeud tile is translated to a code.
/// - 0: No tile (empty square)
/// - 1 .. 26: `a` .. `z`
/// - 27 .. 31: Non-ascii tiles, depending on codec
/// - 64: Blank tile `*` (unassigned)
/// - 65 .. 90: `A` .. `Z` (blank tile assigned to `a`..`z`)
/// - 91 .. 95: Blank tile assigned, depending on codec
///
pub struct Codec {
    codeset: CodeSet,
}

impl CodeSet {
    pub fn new() -> CodeSet {
        let mut encoder = HashMap::new();
        let mut decoder = HashMap::new();
        for (i, ch) in ASCII_LC.chars().enumerate() {
            let uc = ch.to_ascii_uppercase();
            encoder.insert(String::from(ch), i as u8 + 1);
            encoder.insert(String::from(uc), (i as u8 + 1) | IS_WILDCARD);
            decoder.insert(i as u8 + 1, String::from(ch));
            decoder.insert((i as u8 + 1) | IS_WILDCARD, String::from(uc));
        }
        encoder.insert(String::from(" "), EMPTY);
        encoder.insert(String::from("*"), BLANK);
        decoder.insert(EMPTY, String::from(" "));
        decoder.insert(BLANK, String::from("*"));
        CodeSet { encoder, decoder }
    }

    pub fn encode(&self, s: &str) -> Result<Label, CodecError> {
        self.encoder
            .get(&String::from(s))
            .map_or(Err(CodecError::EncodeError(String::from(s))), |label| {
                Ok(*label)
            })
    }

    pub fn decode(&self, label: Label) -> Result<&str, CodecError> {
        self.decoder
            .get(&label)
            .map_or(Err(CodecError::DecodeError(label)), |s| Ok(s))
    }
}

impl Default for Codec {
    fn default() -> Codec {
        Codec::new()
    }
}

impl Codec {
    /// Return a new default `Codec`, for tile sets that can be encoded with plain ascii.
    pub fn new() -> Codec {
        Codec {
            codeset: CodeSet::new(),
        }
    }
    /// Extend the default codec with a list of non-ascii tiles.
    /// Returns extended `Codec`.
    /// ## Examples
    /// ```
    /// use wordfeud_solver::Codec;
    /// let codec = Codec::new().extend(&["ä", "ö", "ü"]);
    /// ```
    pub fn extend(mut self, codes: &[&str]) -> Codec {
        let n = ASCII_LC.chars().count() as u8;
        for (i, s) in codes.iter().enumerate() {
            // TODO check for already present codes
            self.codeset
                .encoder
                .insert(String::from(*s), i as u8 + n + 1);
            self.codeset.encoder.insert(
                String::from(*s).to_uppercase(),
                (i as u8 + n + 1) | IS_WILDCARD,
            );
            self.codeset
                .decoder
                .insert(i as u8 + n + 1, String::from(*s));
            self.codeset.decoder.insert(
                (i as u8 + n + 1) | IS_WILDCARD,
                String::from(*s).to_uppercase(),
            );
        }
        self
    }

    /// Encode string, and return a list of `u8` labels.
    /// ## Errors
    /// An error is returned if the string can not be encoded with the codec.
    /// ## Examples
    /// ```
    /// use wordfeud_solver::{Codec, CodecError};
    /// let codec = Codec::new().extend(&["ä", "ö", "ü"]);
    /// let labels = codec.encode("azAZä *")?;
    /// assert_eq!(labels, vec![1,26,65,90,27,0,64]);
    /// # Ok::<(), CodecError>(())
    /// ```
    pub fn encode(&self, s: &str) -> Result<Vec<Label>, CodecError> {
        s.chars()
            .map(|ch| self.codeset.encode(&String::from(ch)))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Decode labels, and return a list of strings.
    /// ## Errors
    /// An error is returned if the labels can not be decoded with the codec.
    /// ## Examples
    /// ```
    /// use wordfeud_solver::{Codec, CodecError};
    /// let codec = Codec::new().extend(&["ä", "ö", "ü"]);
    /// let labels = &[1,26,65,90,27,0,64];
    /// let decoded = codec.decode(labels)?;
    /// assert_eq!(decoded, &["a","z","A","Z","ä"," ", "*"]);
    /// # Ok::<(), CodecError>(())
    /// ```
    pub fn decode(&self, t: &[Label]) -> Result<Vec<&str>, CodecError> {
        t.iter()
            .map(|label| self.codeset.decode(*label))
            .collect::<Result<Vec<_>, _>>()
    }
}

pub fn encode(s: &str) -> Result<Vec<Label>, CodecError> {
    DEFAULT_CODEC.encode(s)
}

pub fn decode(t: &[Label]) -> Result<Vec<&str>, CodecError> {
    DEFAULT_CODEC.decode(t)
}
