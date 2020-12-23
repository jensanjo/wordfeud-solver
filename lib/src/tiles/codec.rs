use super::{
    codes::{Code, BLANK, EMPTY},
    DIM,
};
use crate::error::Error;
use std::collections::HashMap;
use std::iter::IntoIterator;

/// String corresponding to tile code
pub type Token = String;

/// A list of `Token`'s
pub type Tokens = Vec<Token>;

const NCODE: usize = 256;
const NOCODE: [Option<char>; 2] = [None; 2];

const ASCII_LC: &str = "abcdefghijklmnopqrstuvwxyz";

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct CodeSet {
    encoder: HashMap<String, Code>,
    decoder: Vec<[Option<char>; 2]>,
}

impl CodeSet {
    /// Return a new default codec
    pub fn new(extend: &[&str]) -> CodeSet {
        let mut encoder = HashMap::new();

        for (i, ch) in ASCII_LC.chars().enumerate() {
            let uc = ch.to_ascii_uppercase();
            encoder.insert(String::from(ch), i as u8 + 1);
            encoder.insert(String::from(uc), (i as u8 + 1) | BLANK);
        }
        let n = ASCII_LC.len();
        for (i, s) in extend.iter().enumerate() {
            // TODO check for already present codes
            encoder.insert(String::from(*s), (i + n + 1) as u8);
            encoder.insert(String::from(*s).to_uppercase(), (i + n + 1) as u8 | BLANK);
        }
        encoder.insert(String::from("."), EMPTY);
        encoder.insert(String::from("*"), BLANK);

        let mut decoder = vec![NOCODE; NCODE];
        for (k, &v) in &encoder {
            let mut it = k.chars();
            decoder[v as usize] = [it.next(), it.next()];
        }
        encoder.insert(String::from(" "), EMPTY); // encode both '.' and ' ' to EMPTY, but always decode to '.'
        CodeSet { encoder, decoder }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl Default for Codec {
    fn default() -> Codec {
        Codec::new(&[])
    }
}

impl Codec {
    /// Return a new `Codec` for "a".."z", "*", " ", extended with the non-ascii tiles from `extend`.
    /// ## Examples
    /// ```
    /// use wordfeud_solver::Codec;
    /// let codec = Codec::new(&["ä", "ö", "ü"]);
    /// ```
    pub fn new(extend: &[&str]) -> Codec {
        Codec {
            codeset: CodeSet::new(extend),
        }
    }

    /// A simple tokenizer for single char tiles (does not support spanish)
    fn tokenize(&self, word: &str) -> Tokens {
        word.chars().map(String::from).collect::<Vec<_>>()
    }

    /// Encode string, and return a list of `u8` labels.
    /// ## Errors
    /// An error is returned if the string can not be encoded with the codec.
    /// ## Examples
    /// ```
    /// use wordfeud_solver::{Codec, Error};
    /// let codec = Codec::new(&["ä", "ö", "ü"]);
    /// let labels = codec.encode("azAZä *")?;
    /// assert_eq!(labels, vec![1,26,65,90,27,0,64]);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn encode(&self, word: &str) -> Result<Vec<u8>, Error> {
        let tokens = self.tokenize(word);
        if tokens.len() > DIM {
            return Err(Error::EncodeStringTooLong(String::from(word)));
        }
        let codes = tokens
            .into_iter()
            .map(|token| {
                self.codeset
                    .encoder
                    .get(&token)
                    .copied()
                    .ok_or(Error::EncodeInvalidToken(token))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(codes)
    }
    /// Decode labels, and return a list of strings.
    /// ## Errors
    /// An error is returned if the labels can not be decoded with the codec.
    /// ## Examples
    /// ```
    /// use wordfeud_solver::{Codec, Error};
    /// let codec = Codec::new(&["ä", "ö", "ü"]);
    /// let labels = &[1,26,65,90,27,0,64];
    /// let decoded = codec.decode(labels);
    /// assert_eq!(decoded, &["a","z","A","Z","ä",".", "*"]);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn decode(&self, codes: &[Code]) -> Vec<String> {
        codes
            .iter()
            .map(|&code| {
                let chars = self.codeset.decoder[code as usize];
                let mut s = String::new();
                s.push(chars[0].unwrap());
                if let Some(ch) = chars[1] {
                    s.push(ch);
                }
                s
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_tokenize() {
        let codec = Codec::default();
        let word = "wordfeud";
        let tokens = codec.tokenize(word);
        assert_eq!(tokens, vec!["w", "o", "r", "d", "f", "e", "u", "d"]);
    }

    #[test]
    fn test_encode() {
        let codec = Codec::default();
        let word = "azAZ*";
        let codes = codec.encode(word).unwrap();
        assert_eq!(codes, vec![1, 26, 65, 90, 64]);
        println!("{:?}", codes);
    }

    #[test]
    #[should_panic(expected = "EncodeInvalidToken")]
    fn test_encode_error() {
        let codec = Codec::default();
        let word = "Illegal!";
        let codes = codec.encode(word).unwrap();
        println!("{:?}", codes);
    }
}
