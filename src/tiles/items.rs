use super::{list::ItemList, list::Items, Cell, Letter, List, Tile};

/// A collection of [`Tile`](crate::Tile).
pub type Word = ItemList<Tile>;

/// A collection of [`Cell`](crate::Cell).
pub type Row = ItemList<Cell>;

/// A collection of [`Letter`](crate::Letter).
pub type Letters = ItemList<Letter>;

impl Letters {
    pub fn blank() -> Letters {
        let mut letters = Letters::new();
        letters.push(Letter::blank());
        letters
    }

    pub fn remove(&self, pos: usize) -> Letters {
        let mut w = *self;
        w.0.remove(pos);
        w
    }

    // pub fn contains
}

impl Row {
    /// check if row is a single empty cell
    pub fn is_empty_cell(&self) -> bool {
        self.len() == 1 && self[0].is_empty()
    }

    /// Returns the beginning and the end of the word at position i given that
    /// a character would be placed in i.
    pub(crate) fn start_end(&self, i: usize) -> (usize, usize) {
        let start = self.0[..i]
            .iter()
            .rposition(Cell::is_empty)
            .map_or(0, |p| p + 1);

        let end = self.0[i + 1..]
            .iter()
            .position(Cell::is_empty)
            .map_or(self.len(), |p| p + i + 1);

        (start, end)
    }
    /// Return surrounding word at index `i`.
    pub(crate) fn surrounding_word(&self, i: usize) -> Row {
        let (start, end) = self.start_end(i);
        let inner: Items<Cell> = self.0[start..end]
            .iter()
            .map(|cell| cell.to_letter())
            .collect();
        ItemList(inner)
    }

    /// Replace `from` square to `to` in self[start..end] and return as new Row.
    pub(crate) fn replace(&self, start: usize, end: usize, from: Cell, to: Cell) -> Row {
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
}

impl From<&Row> for Word {
    /// Panics if row contain empty
    fn from(row: &Row) -> Self {
        let items: Items<Tile> = row.iter().map(|cell| cell.tile().unwrap()).collect();
        Self(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Code, Codec, Error, List};
    use std::convert::TryFrom;

    #[test]
    fn test_word() -> Result<(), Error> {
        let tile = Tile::try_from(1);
        println!("{:?}", tile);

        let word = Word::try_from(vec![1, 2, 3])?;
        println!("{:?} {}", word, word.len());
        Ok(())
    }
    #[test]
    fn test_row() -> Result<(), Error> {
        let cell = Cell::try_from(1);
        println!("{:?}", cell);

        let row = Row::try_from(vec![0, 1, 65])?;
        println!("{:?} {} {}", row, row.len(), row.is_empty_cell());
        for cell in row {
            if let Some(tile) = cell.tile() {
                println!("{:?} {}", tile, tile.label());
            }
        }
        Ok(())
    }

    #[test]
    fn test_letters() -> Result<(), Error> {
        let codec = Codec::default();
        let letter = Letter::blank();
        assert!(letter.is_blank());

        let letters = Letters::try_from(codec.encode("rust")?)?;
        assert_eq!(letters.codes(), vec![18, 21, 19, 20]);
        assert_eq!(letters.len(), 4);

        // slice index works
        let s = &letters[0..2];
        assert!(s.contains(&letters[1]));

        // remove works
        let letters = letters.remove(2);
        assert_eq!(letters.codes(), vec![18, 21, 20]);
        Ok(())
    }

    #[test]
    fn test_iter() -> Result<(), Error> {
        let word = Word::try_from(vec![1, 2, 3])?;
        let codes: Vec<Code> = word.codes();
        assert_eq!(codes, vec![1, 2, 3]);
        Ok(())
    }
}
