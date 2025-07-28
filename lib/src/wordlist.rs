mod matches;
mod trievec;

use self::trievec::TrieVec;
use crate::labelset::{Label, LabelSet};
pub use crate::tiles::{Item, ItemList, List, Row};
use crate::Codec;
use crate::Error;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::fmt;
use std::fs::read_to_string;
use tinyvec::ArrayVec;

/// A set of letters
pub type LetterSet = LabelSet;

/// The dimension of wordfeud board: N x N squares
pub const N: usize = 15;

type RowCache = [(LetterSet, bool); N + 1];

/// A list of 0..N (possible letters, connected) tuples.
pub type RowData = ArrayVec<RowCache>;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// A trie data structure that holds all the possible words.
pub struct Wordlist {
    /// List of nodes in trie. Each node is a tuple with the index of the first
    /// child node, and a `CharSet` with the labels of all child nodes.
    pub nodes: Vec<(u32, LabelSet)>,
    /// List of labels.
    pub labels: Vec<Label>,
    /// List indicating terminal nodes
    pub terminal: Vec<bool>,
    /// Path of the wordfile used to build the wordlist.
    /// Empty if the wordlist is not build from a file.
    pub wordfile: String,
    /// The set of all characters used in the wordlist.
    pub all_labels: LabelSet,
    /// The number of words in the wordlist
    pub word_count: usize,
    /// The number of nodes in the wordlist.
    pub node_count: usize,
    /// Encode words to/from labelvec
    pub codec: Codec,
}

impl fmt::Display for Wordlist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<Wordlist: {} words, {} nodes from '{}'>",
            self.word_count, self.node_count, self.wordfile
        )
    }
}

impl From<TrieVec<Label>> for Wordlist {
    fn from(trie: TrieVec<Label>) -> Self {
        let mut nodes: Vec<(u32, LabelSet)> = Vec::new();
        let mut labels: Vec<Label> = Vec::new();
        let mut terminal: Vec<bool> = Vec::new();
        let mut word_count = 0;
        let mut node_count = 0;
        let mut all_labels: LabelSet = LabelSet::new();
        let codec = Codec::default(); // TODO

        let mut i: usize = 0;
        let mut queue = VecDeque::new();
        queue.push_back((&trie, i, 0));
        while let Some((node, index, k)) = queue.pop_front() {
            let mut ls = LabelSet::new();
            for (label, t) in node.children() {
                ls.insert(*label);
                all_labels.insert(*label);
                queue.push_back((t, i, *label));
            }
            if node.terminal() {
                word_count += 1;
            }
            nodes.push((0, ls));
            terminal.push(node.terminal());
            labels.push(k);
            if nodes[index].0 == 0 {
                nodes[index].0 = i as u32;
            }
            i += 1;
            node_count += 1;
        }
        Wordlist {
            nodes,
            labels,
            terminal,
            wordfile: String::new(),
            all_labels,
            word_count,
            node_count,
            codec,
        }
    }
}

pub struct IteratorChildren<'a> {
    wordlist: &'a Wordlist,
    range: Option<(usize, usize)>,
    i: Option<usize>,
}

impl<'a> Iterator for IteratorChildren<'a> {
    type Item = (Label, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((s, e)) = self.range {
            if self.i.is_none() {
                self.i = Some(s);
            }
            if let Some(i) = self.i {
                if i <= e {
                    let label = self.wordlist.labels[i];
                    self.i = Some(i + 1);
                    return Some((label, i));
                }
            }
        }
        None
    }
}

impl Wordlist {
    /// Read the wordlist from a file. The file must be encoded in utf-8 and
    /// have one word per line.
    /// ## Errors
    /// Fails if the wordlist can not be read, or a word can not be encoded.
    pub fn from_file(wordfile: &str, codec: &Codec) -> Result<Wordlist, Error> {
        let mut builder = TrieVec::new();
        read_to_string(wordfile)
            .map_err(|source| Error::ReadError {
                path: String::from(wordfile),
                source,
            })?
            .lines()
            .map(str::trim)
            .map(|word| codec.encode(&word).map(|labels| builder.insert(&labels)))
            .collect::<Result<Vec<()>, Error>>()?;
        let mut wordlist = Wordlist::from(builder);
        wordlist.wordfile = String::from(wordfile);
        wordlist.codec = codec.clone();
        Ok(wordlist)
    }

    /// Build a wordlist from a list of words.
    /// ## Errors
    /// If a word can not be encoded with the given `codec`.
    pub fn from_words(words: &[&str], codec: &Codec) -> Result<Wordlist, Error> {
        let mut builder = TrieVec::new();
        for &word in words {
            builder.insert(codec.encode(word)?);
        }
        let mut wordlist = Wordlist::from(builder);
        wordlist.codec = codec.clone();
        Ok(wordlist)
    }

    #[cfg(feature = "bincode")]
    /// Deserialize the wordlist from a bincoded file.
    /// ## Errors
    /// - If the wordlist can not be read.
    /// - If the contents can not be deserialized
    pub fn deserialize_from(wordfile: &str) -> Result<Wordlist, Error> {
        use std::fs::File;
        use std::io::BufReader;
        let file = File::open(wordfile).map_err(|source| Error::ReadError {
            path: String::from(wordfile),
            source,
        })?;
        let reader = BufReader::new(file);
        let mut wordlist: Wordlist = bincode::deserialize_from(reader)
            .map_err(|_| Error::WordfileDeserializeError(String::from(wordfile)))?;
        wordlist.wordfile = String::from(wordfile);
        Ok(wordlist)
    }

    /// Encode a word with our `codec`.
    /// ## Errors
    /// If the word can not be encoded.
    pub fn encode<T: Item>(&self, word: &str) -> Result<ItemList<T>, Error> {
        ItemList::<T>::try_from(self.codec.encode(word)?)
    }

    /// Decode `labels` with our [`codec`](Wordlist::codec), and return the result as `String`.
    /// ## Errors
    /// If the labels can not be decoded.
    pub fn decode<T: Item>(&self, items: ItemList<T>) -> String {
        self.codec.decode(&items.codes()).join("")
    }

    /// Return the start and end index of the child nodes of node `i`,
    /// or None if node is empty.
    pub fn range_children(&self, i: usize) -> Option<(usize, usize)> {
        let (start, labels) = &self.nodes[i];
        let s = *start as usize;
        let len = labels.len();
        match len {
            0 => None,
            n => Some((s, s + n - 1)),
        }
    }

    /// Iterate over the children of node `i`.
    pub fn iter_children(&self, i: usize) -> IteratorChildren {
        IteratorChildren {
            wordlist: self,
            range: self.range_children(i),
            i: None,
        }
    }

    /// Get the index of child with `label` for node `i` if present.
    pub fn get(&self, i: usize, label: Label) -> Option<usize> {
        let (start, labels) = &self.nodes[i];
        if let Some(index) = labels.index_of(label) {
            return Some(*start as usize + index);
        }
        None
    }

    /// Returns true if `word` is in wordlist
    pub fn is_word<K: AsRef<[Label]>>(&self, word: K) -> bool {
        let mut i = 0;
        for c in word.as_ref() {
            let (start, labels) = &self.nodes[i];
            if let Some(pos) = labels.index_of(*c) {
                i = *start as usize + pos;
            } else {
                return false;
            }
        }
        self.terminal[i]
    }

    pub fn connected_row(&self, row: &Row) -> RowData {
        RowData::from_array_len([(self.all_labels, true); N + 1], row.len())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tiles::Word;

    const WORDS: &[&str] = &[
        "af", "ah", "al", "aar", "aas", "bi", "bo", "bar", "bes", "bel", "belt",
    ];

    fn test_wordlist() -> Wordlist {
        Wordlist::from_words(WORDS, &Codec::default()).unwrap()
    }

    #[test]
    fn test_range() {
        let wordlist = test_wordlist();
        println!("{:?}", wordlist);
        assert_eq!(wordlist.word_count, 11);
        assert_eq!(wordlist.node_count, 17);
        assert_eq!(wordlist.range_children(0), Some((1, 2)));
        assert_eq!(wordlist.range_children(1), Some((3, 6)));
        assert_eq!(wordlist.range_children(4), None);
    }

    #[test]
    fn test_terminal() {
        let wordlist = test_wordlist();
        assert!(wordlist.terminal[4]);
        assert!(!wordlist.terminal[0]);
    }

    #[test]
    fn test_is_word() {
        let wordlist = test_wordlist();
        for &word in WORDS {
            let w: Word = wordlist.encode(word).unwrap();
            assert!(wordlist.is_word(w.codes()));
        }
    }
}
