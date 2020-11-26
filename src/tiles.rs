use std::convert::From;
use std::convert::TryFrom;
use std::fmt;
use std::num::NonZeroU8;
use std::ops;
use std::slice::Iter;
use tinyvec::ArrayVec;

use crate::codec::{self, Label, BLANK, IS_WILDCARD, LETTER_MASK};
use crate::Error;

const N: usize = 15;

type TileVec = ArrayVec<[Tile; N + 1]>;
type Square = Option<Tile>;
type RowVec = ArrayVec<[Square; N + 1]>;

#[derive(Debug, Clone, Copy, PartialEq)]
/// A wrapper around a (nonzero) `u8` tile code.
pub struct Tile(NonZeroU8);

impl Tile {
    /// Returns a new blank tile
    fn new() -> Tile {
        Tile::from_label(BLANK)
    }

    /// Return a tile with value `n`. A tile can not have value 0.
    /// ## Panics
    /// If `n` is 0.  
    fn from_label(n: u8) -> Tile {
        let label = NonZeroU8::new(n).expect("label can't be 0");
        Tile(label)
    }

    fn as_letter(&self) -> Tile {
        Tile::from_label(self.label())
    }

    /// Return a wildcard tile with for tile code `n`.
    /// ## Example
    /// ```
    /// use wordfeud_solver::Tile;
    /// let tile = Tile::wildcard_from_letter(1);
    /// assert_eq!(tile.get(), 65);
    /// ```
    pub fn wildcard_from_letter(n: u8) -> Tile {
        Tile::from_label(n | IS_WILDCARD)
    }

    /// Get code value for tile.
    pub fn get(&self) -> Label {
        self.0.get()
    }

    /// Get label for tile, ignoring the wildcard attribute.
    /// ## Example
    /// ```
    /// use wordfeud_solver::Tile;
    /// let tile = Tile::wildcard_from_letter(1);
    /// assert_eq!(tile.get(), 65);
    /// assert_eq!(tile.label(), 1);
    /// ```
    pub fn label(&self) -> Label {
        self.get() & LETTER_MASK
    }

    /// Check if the tile is a blank.
    pub fn is_blank(&self) -> bool {
        self.get() == BLANK
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Default, PartialEq)]
/// A wrapper around a vec of squares possibly containing tiles.
pub struct Row(RowVec);

impl Row {
    /// Return a new empty row.
    pub fn new() -> Row {
        Row(RowVec::new())
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn push(&mut self, square: Square) {
        self.0.push(square);
    }

    pub(crate) fn iter(&self) -> Iter<Square> {
        self.0.iter()
    }

    /// Return the contents as a vec of `u8` tile codes (possibly 0).
    /// ## Examples
    /// ```
    /// use wordfeud_solver::Row;
    /// let row = Row::from(vec![0,1,2]);
    /// let v = row.as_vec();
    /// assert_eq!(v, vec![0,1,2]);
    /// ```
    pub fn as_vec(&self) -> Vec<Label> {
        Vec::<Label>::from(self)
    }

    /// Returns the beginning and the end of the word at position i given that
    /// a character would be placed in i.
    pub(crate) fn start_end(&self, i: usize) -> (usize, usize) {
        let start = self.0[..i]
            .iter()
            .rposition(Option::is_none)
            .map_or(0, |p| p + 1);

        let end = self.0[i + 1..]
            .iter()
            .position(Option::is_none)
            .map_or(self.len(), |p| p + i + 1);

        (start, end)
    }

    /// Return surrounding word at index `i`.
    pub(crate) fn surrounding_word(&self, i: usize) -> Row {
        let (start, end) = self.start_end(i);
        let row: RowVec = self.0[start..end]
            .iter()
            .map(|square| square.map_or(None, |t| Some(t.as_letter())))
            .collect::<RowVec>();
        Row(row)
    }

    /// Replace `from` square to `to` in self[start..end] and return as new Row.
    pub(crate) fn replace(&self, start: usize, end: usize, from: Square, to: Square) -> Row {
        debug_assert!(start < end);
        let mut row = Row::new();
        for i in start..end {
            if self.0[i] == from {
                row.push(to);
            } else {
                row.push(self.0[i]);
            }
        }
        row
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 1 && self[0].is_none()
    }
}

impl From<Vec<u8>> for Row {
    fn from(labels: Vec<u8>) -> Self {
        let t: RowVec = labels
            .iter()
            .map(|e| {
                if *e == 0 {
                    None
                } else {
                    Some(Tile::from_label(*e))
                }
            })
            .collect();
        // It could just be: unsafe { std::mem::transmute(labels) }
        Self(t)
    }
}

impl From<&Row> for Vec<u8> {
    fn from(row: &Row) -> Self {
        row.iter()
            .map(|t| match t {
                Some(tile) => tile.get(),
                None => 0,
            })
            .collect()
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let labels = self.as_vec();
        write!(f, "{}", codec::decode(&labels).unwrap().join("")) // TODO
    }
}

impl fmt::Debug for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Row(\"{}\")", self.to_string())
    }
}

impl TryFrom<&str> for Row {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let labels = codec::encode(s)?;
        Ok(Row::from(labels))
    }
}

impl ops::Index<usize> for Row {
    type Output = Square;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl ops::IndexMut<usize> for Row {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
/// A wrapper around a vec of (non-empty) `Tile`'s.
/// Used to represent [`Word`](crate::Word) and [`Letters`](crate::Letters).
pub struct Tiles(TileVec);

impl Tiles {
    /// Return new empty `Tiles`.
    pub fn new() -> Tiles {
        Tiles(TileVec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, tile: Tile) {
        self.0.push(tile);
    }

    pub fn iter(&self) -> Iter<Tile> {
        self.0.iter()
    }

    /// Get a single blank tile (wildcard).
    pub fn blank() -> Tiles {
        let mut tiles = Tiles::new();
        tiles.push(Tile::new());
        tiles
    }

    pub fn as_vec(&self) -> Vec<Label> {
        Vec::<Label>::from(self)
    }

    pub fn remove(&self, pos: usize) -> Tiles {
        let mut w = *self;
        w.0.remove(pos);
        w
    }
}

impl From<Vec<u8>> for Tiles {
    /// Panics if labels contain zero (empty)
    fn from(labels: Vec<u8>) -> Self {
        let t: TileVec = labels.iter().map(|t| Tile::from_label(*t)).collect();
        Self(t)
    }
}

impl From<&Tiles> for Vec<u8> {
    fn from(tiles: &Tiles) -> Self {
        tiles.iter().map(Tile::get).collect()
    }
}

impl From<&Row> for Tiles {
    /// Panics if row contain empty
    fn from(row: &Row) -> Self {
        let t: TileVec = row.iter().map(|t| t.unwrap()).collect();
        Tiles(t)
    }
}

impl fmt::Display for Tiles {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let labels = self.as_vec();
        write!(f, "{}", codec::decode(&labels).unwrap().join("")) // TODO
    }
}

impl TryFrom<&str> for Tiles {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let labels = codec::encode(s)?;
        Ok(Tiles::from(labels))
    }
}

impl ops::Index<usize> for Tiles {
    type Output = Tile;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl ops::Index<ops::Range<usize>> for Tiles {
    type Output = [Tile];
    fn index<'a>(&'a self, range: ops::Range<usize>) -> &'a Self::Output {
        &self.0[range]
    }
}

// impl Deref for Tiles {
//     type Target = TileVec;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for Tiles {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_nonzero() {
        let n = Tile::new();
        println!("{:?} {}", n, std::mem::size_of_val(&n));
    }

    #[test]
    fn test_row() -> Result<()> {
        let row = Row::try_from("abzAZ ")?;
        for t in row.iter() {
            println!("{:?}", t);
        }
        println!("{:?} \"{}\"", row.as_vec(), row);
        let row = Row::try_from("a    ")?;
        println!("{:?} \"{}\"", row, row);
        Ok(())
    }

    #[test]
    fn test_tiles() -> Result<()> {
        let tiles = Tiles::try_from("abzAZ")?;
        for t in tiles.iter() {
            println!("{:?}", t);
        }
        println!("{:?} \"{}\"", tiles.as_vec(), tiles);
        Ok(())
    }

    #[test]
    fn test_tiles_from_row() -> Result<()> {
        let row = Row::try_from("Aap")?;
        let tiles = Tiles::from(&row);
        println!("{:?} {:?} \"{}\"", row, tiles, tiles);
        Ok(())
    }

    #[test]
    fn test_start_end() -> Result<()> {
        assert_eq!(Row::try_from("  aap noot ")?.start_end(1), (1, 5));
        assert_eq!(Row::try_from("aap")?.start_end(1), (0, 3));
        assert_eq!(Row::try_from("  aap noot ")?.start_end(6), (6, 10));
        assert_eq!(Row::try_from("    t     c   f")?.start_end(13), (13, 15));
        assert_eq!(Row::try_from("    t     c   f")?.start_end(11), (10, 12));
        Ok(())
    }

    #[test]
    fn test_surrounding_word() -> Result<()> {
        let row = Row::try_from("  aap noot ")?;
        assert_eq!(row.surrounding_word(1).to_string(), " aap");
        assert_eq!(row.surrounding_word(2).to_string(), "aap");
        assert_eq!(row.surrounding_word(6).to_string(), "noot");
        Ok(())
    }
}
