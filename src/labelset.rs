use crate::codec;
use bitintr::{Bzhi, Popcnt};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::fmt;
use std::iter::FromIterator;
use std::iter::Iterator;

pub type Label = u8;

/// A bitset representing labels present in a `wordlist` node
#[derive(Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LabelSet(u32);

impl LabelSet {
    pub fn new() -> LabelSet {
        LabelSet(0)
    }

    pub fn contains(&self, label: Label) -> bool {
        let v = label;
        self.0 & (1 << v) != 0
    }

    pub fn insert(&mut self, label: Label) -> bool {
        let v = label;
        assert!(v < 32);
        let r = (self.0 & (1 << v)) != 0;
        self.0 |= 1 << v;
        r
    }

    pub fn len(&self) -> usize {
        self.0.popcnt() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> IteratorLabelSet {
        IteratorLabelSet::new(self.0)
    }

    /// Return the bit position corresponding to char if present.
    pub fn index_of(&self, label: Label) -> Option<usize> {
        if !self.contains(label) {
            return None;
        }
        let v = label as u32;
        Some(self.0.bzhi(v).popcnt() as usize)
    }
}

impl fmt::Display for LabelSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let labels: Vec<u8> = self.iter().collect();
        write!(f, "{}", codec::decode(&labels).unwrap().join(""))
    }
}

impl fmt::Debug for LabelSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self
            .iter()
            .map(|label| format!("{}", label))
            .collect::<Vec<String>>()
            .join(",");
        write!(f, "{{{}}}", s)
    }
}

pub struct IteratorLabelSet {
    count: u32,
    value: u32,
}

impl IteratorLabelSet {
    pub fn new(value: u32) -> IteratorLabelSet {
        IteratorLabelSet { count: 0, value }
    }
}

impl Iterator for IteratorLabelSet {
    type Item = Label;
    fn next(&mut self) -> Option<Label> {
        while self.count < 32 {
            let i = self.count;
            self.count += 1;
            let v = 1 << i;
            if self.value & v != 0 {
                return Some(i as Label);
            }
        }
        None
    }
}

impl FromIterator<u8> for LabelSet {
    fn from_iter<I: IntoIterator<Item = u8>>(iter: I) -> Self {
        let mut c = LabelSet::new();
        for i in iter {
            c.insert(i);
        }
        c
    }
}

impl From<Vec<u8>> for LabelSet {
    fn from(v: Vec<u8>) -> Self {
        LabelSet::from_iter(v.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_labelset() {
        let mut labels = LabelSet::new();
        for &n in &[2, 25, 2, 1] {
            labels.insert(n);
        }
        for &n in &[1, 2, 25] {
            assert!(labels.contains(n));
        }
        assert!(!labels.contains(5));
        // println!("{} {:?} {}", labels.0, labels, labels);
        // assert!(false);
    }

    #[test]
    fn test_labelset_iterator() {
        let labels = LabelSet::from(vec![0u8, 1, 4, 5, 7, 8, 10, 12, 14, 15]);
        println!("{:?}", labels);
    }

    #[test]
    fn test_index() {
        let labels = LabelSet::from(vec![0u8, 1, 4, 5, 7, 8, 10, 12, 14, 15]);
        println!("{:?}", labels);
        assert_eq!(labels.index_of(0), Some(0));
        assert_eq!(labels.index_of(15), Some(9));
        assert_eq!(labels.index_of(2), None);
        assert_eq!(labels.len(), 10);
    }
}
