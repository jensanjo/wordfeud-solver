#![allow(dead_code)]
use tinyvec::{ArrayVec, array_vec};

const N: usize = 15;

trait Container<T> where T: Default {
    fn as_vec(&self) -> Vec<T>;

}

struct Cells<T>(ArrayVec<[T; N]>) where T: Default;

#[derive(Default)]
struct Tile(u8);

impl Tile {
    fn new(n: u8) -> Tile {
        Tile(n)
    }
}

type Square = Option<Tile>;
type Row = Cells<Square>;
type Tiles = Cells<Tile>;

impl Row {

}

impl Tiles {
    fn new(tiles: &[u8]) -> Tiles {
        let tiles = ArrayVec::from(tiles);
        Cells::<Tile>(tiles)
    }


}


#[cfg(test)]
mod tests {
    use tinyvec::array_vec;
    use super::*;

    #[test]
    fn test_append() {
        let a = array_vec!([u8; 14] => 1,2,3);
        println!("{:?} {}", a, std::mem::size_of_val(&a));
        let b = { let mut t = a; t.push(2); t};
        println!("{:?} {:?}", a, b);

    }

    #[test]
    fn test_tiles() {
        let t: Tiles = Tiles::new();
    }
}