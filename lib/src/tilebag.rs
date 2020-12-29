#![allow(dead_code)]
use crate::tiles::BLANK;
use crate::{Code, TileSet};
use multiset::HashMultiSet;
use std::convert::From;
use std::ops::Deref;
use std::ops::Sub;

/// Keeps track of the tiles
#[derive(Debug, Clone)]
pub struct TileBag(HashMultiSet<Code>);

impl Deref for TileBag {
    type Target = HashMultiSet<Code>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Sub for TileBag {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl TileBag {
    pub fn new() -> Self {
        Self(HashMultiSet::new())
    }

    fn from_tileset(tileset: &TileSet) -> Self {
        let mut bag = HashMultiSet::new();
        for (code, &(_label, count, _points)) in tileset.tiles.iter().enumerate() {
            if count > 0 {
                bag.insert_times(code as u8, count as usize);
            }
        }
        // add two blanks
        bag.insert_times(BLANK, 2);
        Self(bag)
    }

    fn from_tiles<T: AsRef<[Code]>>(tiles: T) -> TileBag {
        let mut bag = HashMultiSet::new();
        for &tile in tiles.as_ref() {
            bag.insert(tile);
        }
        Self(bag)
    }
}

impl<'a> From<&'a TileSet<'a>> for TileBag {
    fn from(tileset: &TileSet) -> Self {
        TileBag::from_tileset(tileset)
    }
}

impl<T> From<T> for TileBag
where
    T: AsRef<[Code]>,
{
    fn from(tiles: T) -> Self {
        TileBag::from_tiles(tiles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Language;

    #[test]
    fn test_bag() {
        let tileset = TileSet::new(Language::NL);
        let bag = TileBag::from_tileset(&tileset);
        println!("{:?}", bag);
    }
}
