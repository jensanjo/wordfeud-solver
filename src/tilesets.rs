#![allow(dead_code)]
use crate::codec::{Codec, Label};

mod en;
mod nl;
mod se;

/// These languages are supported.
#[derive(Debug, Clone)]
pub enum Language {
    /// English
    EN,
    /// Dutch
    NL,
    /// Swedish
    SE,
}

/// label, count, points
type TileInfo = (&'static str, u32, u32);

/// A tileset for `wordfeud`. It contains the tile distribution for a supported language,
/// and a codec to translate between words and tiles. The tile distributions are specified on the 
/// [Wordfeud.com website](https://wordfeud.com/wf/help/)
#[derive(Debug, Clone)]
pub struct TileSet<'a> {
    language: Language,
    tiles: &'a [TileInfo],
    codec: Codec,
}

impl<'a> TileSet<'a> {
    /// Return a new `TileSet` for language.
    pub fn new(language: Language) -> TileSet<'a> {
        let tiles = match language {
            Language::EN => en::TILESET,
            Language::NL => nl::TILESET,
            Language::SE => se::TILESET,
        };
        // get additional labels past a..z
        let extended: Vec<&str> = tiles[27..].iter().map(|&tile| tile.0).collect();
        let codec = Codec::new().extend(&extended);
        TileSet {
            language,
            tiles,
            codec,
        }
    }

    /// Return the points for tile, or 0 if not found
    pub fn points(&self, tilecode: Label) -> u32 {
        if let Some(&tile) = self.tiles.get(tilecode as usize) {
            return tile.2;
        }
        0
    }

    /// Return the number of tiles with this code in tileset, or 0 if not found
    pub fn count(&self, tilecode: Label) -> u32 {
        if let Some(&tile) = self.tiles.get(tilecode as usize) {
            return tile.1;
        }
        0
    }

    /// Return the number of tiles with this code in tileset, or 0 if not found
    pub fn label(&self, tilecode: Label) -> &'a str {
        if let Some(&tile) = self.tiles.get(tilecode as usize) {
            return tile.0;
        }
        " "
    }

    /// Return the codec for this language
    pub fn codec(&self) -> &Codec {
        &self.codec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn test_tileset() {
        let tileset = TileSet::new(Language::NL);
        println!("{:?}", tileset);
        assert_eq!(tileset.points(0), 0);
        assert_eq!(tileset.points(26), 5);
        assert_eq!(tileset.count(5), 18);
        assert_eq!(tileset.label(5), "e");
    }

    #[test]
    fn test_codec() -> Result<(), Error> {
        let tileset = TileSet::new(Language::SE);
        let codec = tileset.codec();
        assert_eq!(codec.encode("azåAZ*")?, &[1, 26, 27, 65, 90, 64]);
        assert_eq!(
            codec.decode(&[1, 26, 27, 28, 29, 65, 90, 91, 92, 93, 64])?,
            &["a", "z", "å", "ä", "ö", "A", "Z", "Å", "Ä", "Ö", "*"]
        );
        Ok(())
    }
}
