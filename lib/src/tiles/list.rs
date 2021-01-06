use super::DIM;
use super::{Code, Item};
use crate::error::Error;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::iter::{FromIterator, IntoIterator};
use std::ops::{Index, IndexMut, Range};
use std::slice::Iter;
use tinyvec::{ArrayVec, ArrayVecIterator};

/// common trait for a list of [`Item`](crate::Item)
pub trait List:
    Debug
    + Default
    + Clone
    + Copy
    + Index<usize>
    + IndexMut<usize>
    + Index<Range<usize>>
    + PartialEq
    + Eq
{
    type Item;
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn push(&mut self, item: Self::Item);

    fn iter(&self) -> Iter<Self::Item>;
}

pub(super) type Items<T> = ArrayVec<[T; DIM]>;

/// A wrapper around a list of `Item`.
/// Used to represent [`Word`](crate::Word), [`Letters`](crate::Letters) and [`Row`](crate::Row).
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct ItemList<T: Item>(pub Items<T>);

impl<T: Item> Index<usize> for ItemList<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: Item> IndexMut<usize> for ItemList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T: Item> Index<Range<usize>> for ItemList<T> {
    type Output = [T];
    fn index(&self, range: Range<usize>) -> &Self::Output {
        &self.0[range]
    }
}

impl<T: Item> ItemList<T> {
    pub fn new() -> ItemList<T> {
        ItemList::<T>(Items::new())
    }

    pub fn codes(&self) -> Vec<Code> {
        self.into_iter().map(T::into).collect()
    }
}

impl<T: Item> List for ItemList<T> {
    type Item = T;
    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn push(&mut self, item: Self::Item) {
        self.0.push(item);
    }

    fn iter(&self) -> Iter<Self::Item> {
        self.0.iter()
    }
}

impl<T: Item> FromIterator<T> for ItemList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        ItemList::<T>(iter.into_iter().collect())
    }
}

impl<T: Item> IntoIterator for ItemList<T> {
    type Item = T;
    type IntoIter = ArrayVecIterator<[T; DIM]>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Item> TryFrom<Vec<u8>> for ItemList<T> {
    type Error = Error;
    fn try_from(codes: Vec<u8>) -> Result<Self, Error> {
        codes.into_iter().map(T::try_from).collect()
    }
}
