#![allow(dead_code)]
use crate::codec::{Codec, Label};
// use anyhow::Result;

mod en;
mod nl;
mod se;

/// label, count, points
type TileInfo = (&'static str, u32, u32);

#[derive(Debug, Clone)]
pub struct TileSet<'a> {
    name: &'a str,
    tiles: &'a [TileInfo],
    codec: Codec,
}

impl<'a> TileSet<'a> {
    pub fn new(name: &'a str) -> TileSet<'a> {
        let tiles = match name {
            "en" => en::TILESET,
            "nl" => nl::TILESET,
            "se" => se::TILESET,
            _ => panic!(format!("Invalid tileset '{}'", name)), // TODO return result
        };
        // get additional labels past a..z
        let extended: Vec<&str> = tiles[27..].iter().map(|&tile| tile.0).collect();
        let codec = Codec::new().extend(&extended);
        TileSet { name, tiles, codec }
    }

    // return the points for tile, or 0 if not found
    pub fn points(&self, tilecode: Label) -> u32 {
        if let Some(&tile) = self.tiles.get(tilecode as usize) {
            return tile.2;
        }
        0
    }

    // return the number of tiles with this code in tileset, or 0 if not found
    pub fn count(&self, tilecode: Label) -> u32 {
        if let Some(&tile) = self.tiles.get(tilecode as usize) {
            return tile.1;
        }
        0
    }

    // return the number of tiles with this code in tileset, or 0 if not found
    pub fn label(&self, tilecode: Label) -> &'a str {
        if let Some(&tile) = self.tiles.get(tilecode as usize) {
            return tile.0;
        }
        " "
    }

    pub fn codec(&self) -> &Codec {
        &self.codec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_tileset() {
        let tileset = TileSet::new("nl");
        println!("{:?}", tileset);
        assert_eq!(tileset.points(0), 0);
        assert_eq!(tileset.points(26), 5);
        assert_eq!(tileset.count(5), 18);
        assert_eq!(tileset.label(5), "e");
    }

    #[test]
    fn test_codec() -> Result<()> {
        let tileset = TileSet::new("se");
        let codec = tileset.codec();
        assert_eq!(codec.encode("azåAZ*")?, &[1, 26, 27, 65, 90, 64]);
        assert_eq!(
            codec.decode(&[1, 26, 27, 28, 29, 65, 90, 91, 92, 93, 64])?,
            &["a", "z", "å", "ä", "ö", "A", "Z", "Å", "Ä", "Ö", "*"]
        );
        Ok(())
    }
}
