use crate::labelset::LabelSet;
use crate::tiles::{Row, Tile, Tiles};
use crate::wordlist::{Letters, RowData, Word, Wordlist, N};
use std::collections::VecDeque;
use std::iter::Iterator;

#[derive(Debug)]
pub struct Matches<'a> {
    wordlist: &'a Wordlist,
    row: Row,
    rowdata: &'a RowData,
    child_iter: VecDeque<Args>,
}

#[derive(Debug)]
struct Args {
    node: usize,
    pos: usize,
    letters: Letters,
    word: Tiles,
    next: Option<Tile>,
    connecting: bool,
    extending: bool,
}

impl<'a> Iterator for Matches<'a> {
    type Item = Word;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut args) = self.child_iter.pop_front() {
            let (node, pos) = (args.node, args.pos);
            if let Some(tile) = args.next {
                args.word.push(tile);
            }
            if pos == self.row.len() {
                return self.next();
            };
            if let Some(tile) = self.row[pos] {
                if let Some(child) = self.wordlist.get(node, tile.label()) {
                    self.child_iter.push_back(Args {
                        node: child,
                        pos: pos + 1,
                        letters: args.letters,
                        word: args.word, //.append(tile),
                        next: Some(tile),
                        connecting: true,
                        extending: args.extending,
                    });
                }
            } else {
                // empty square at row[pos]
                if pos < self.row.len() - 1 {
                    let (valid_chars, connected) = &self.rowdata[pos];
                    for (i, &letter) in args.letters.iter().enumerate() {
                        if !letter.is_blank() && !valid_chars.contains(letter.label()) {
                            continue;
                        }
                        if args.letters[0..i].contains(&letter) {
                            continue;
                        }
                        if letter.is_blank() {
                            // expand wildcard
                            let next_letters = args.letters.remove(i);
                            for (wc, child) in self.wordlist.iter_children(node) {
                                if valid_chars.contains(wc) {
                                    let tile = Tile::wildcard_from_letter(wc);
                                    self.child_iter.push_back(Args {
                                        node: child,
                                        pos: pos + 1,
                                        letters: next_letters,
                                        word: args.word, //.append(tile),
                                        next: Some(tile),
                                        connecting: args.connecting || *connected,
                                        extending: true,
                                    });
                                }
                            }
                        } else {
                            // normal letter
                            if let Some(child) = self.wordlist.get(node, letter.label()) {
                                let next_letters = args.letters.remove(i);
                                self.child_iter.push_back(Args {
                                    node: child,
                                    pos: pos + 1,
                                    letters: next_letters,
                                    word: args.word, //.append(letter),
                                    next: Some(letter),
                                    connecting: args.connecting || *connected,
                                    extending: true,
                                });
                            }
                        }
                    }
                }
                if self.wordlist.terminal[node]
                    && args.connecting
                    && args.extending
                    && args.word.len() > 1
                {
                    return Some(args.word);
                }
            }
        }
        None
    }
}

impl<'a> Matches<'a> {
    fn new(wordlist: &'a Wordlist, row: Row, rowdata: &'a RowData, args: Args) -> Matches<'a> {
        let mut child_iter = VecDeque::new();
        child_iter.push_back(args);
        Matches {
            wordlist,
            row,
            rowdata,
            child_iter,
        }
    }
}

impl Wordlist {
    /// Return a list of matching words. TODO.
    pub fn matches<'a>(
        &'a self,
        node: usize,
        row: Row,
        rowdata: &'a RowData,
        pos: usize,
        letters: &Letters,
    ) -> Matches {
        let args = Args {
            node,
            pos,
            letters: *letters,
            word: Word::new(),
            next: None,
            connecting: false,
            extending: false,
        };
        Matches::new(self, row, rowdata, args)
    }

    pub fn words<'a>(
        &'a self,
        row: &'a Row,
        rowdata: &'a RowData,
        letters: &'a Letters,
        maxdist: Option<usize>,
    ) -> impl Iterator<Item = (usize, Word)> + 'a {
        let mut row = *row;
        row.push(None); //.append(None); // extend row with an empty square

        // calculate possible start locations in row
        let maxdist = maxdist.unwrap_or_else(|| letters.len());
        let mut indices = Vec::new();
        for pos in 0..row.len() {
            if pos > 0 && row[pos - 1].is_some() {
                continue;
            }
            // find next connected
            let dist = rowdata[pos..]
                .iter()
                .position(|(_, connected)| *connected)
                .unwrap_or(N);
            if dist > maxdist {
                continue;
            }
            indices.push(pos);
        }

        indices.into_iter().flat_map(move |pos| {
            self.matches(0, row, rowdata, pos, &letters)
                .map(move |word| (pos, word))
        })
    }

    /// Return a set of characters that go in the first empty position in `word`
    pub fn get_legal_characters(&self, word: &Row) -> LabelSet {
        if word.is_empty() {
            self.all_labels
        } else {
            let mut chars = LabelSet::new();
            if let Some(i) = word.iter().position(|&t| t.is_none()) {
                let rowdata = self.connected_row(word);
                let mut row = *word;
                row.push(None);
                let letters = Letters::blank();
                let matches = self.matches(0, row, &rowdata, 0, &letters);
                for m in matches {
                    let tile = m[i];
                    chars.insert(tile.label());
                }
                chars
            } else {
                LabelSet::new()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::codec::Codec;
    use anyhow::Result;
    use std::collections::HashSet;
    use std::convert::TryFrom;

    const WORDS: &[&str] = &[
        "af", "ah", "al", "aar", "aas", "bi", "bo", "bar", "bes", "bel", "belt",
    ];

    fn test_match_helper(letters: &str, row: &str, pos: usize, expected: &[&str]) -> Result<()> {
        let wordlist = Wordlist::from_words(WORDS, &Codec::default()).unwrap();
        let row = Row::try_from(row)?;
        let rowdata: RowData = wordlist.connected_row(&row);
        let letters = Letters::try_from(letters)?;
        let words = wordlist.matches(0, row, &rowdata, pos, &letters);
        let words: Vec<String> = words.map(|w| w.to_string()).collect();
        println!("{:?}", words);
        assert_eq!(words, expected);
        Ok(())
    }

    #[test]
    fn test_matches_1() -> Result<()> {
        let letters = "abel";
        let expected = &["al", "bel"];
        test_match_helper(letters, "    ", 0, expected)?;
        Ok(())
    }

    #[test]
    fn test_matches_2() -> Result<()> {
        let letters = "ab*";
        // TODO: word order is not alphabetic
        let expected = &["aF", "aH", "aL", "bI", "bO", "baR"];
        test_match_helper(letters, "    ", 0, expected)?;
        Ok(())
    }

    #[test]
    fn test_matches_3() -> Result<()> {
        // Read some words in the wordlist and verify the word count and character set
        test_match_helper("*", "a  ", 0, &["aF", "aH", "aL"])?;
        Ok(())
    }

    #[test]
    fn test_matches_4() -> Result<()> {
        // Read some words in the wordlist and verify the word count and character set
        test_match_helper("ab*", "    t     c   f ", 13, &["af", "Af"])?;
        Ok(())
    }

    #[test]
    fn test_words() -> Result<()> {
        // Read some words in the wordlist and verify the word count and character set
        let wordlist = Wordlist::from_words(WORDS, &Codec::default()).unwrap();
        assert_eq!(wordlist.word_count, WORDS.len());
        let all_labels: Vec<u8> = wordlist.all_labels.iter().collect();
        assert_eq!(all_labels, vec![1, 2, 5, 6, 8, 9, 12, 15, 18, 19, 20]);

        let row = Row::try_from("    t     c   f")?;
        let rowdata = wordlist.connected_row(&row);
        let letters = Letters::try_from("ab*")?;

        let words = wordlist
            .words(&row, &rowdata, &letters, None)
            .collect::<Vec<_>>();
        for (pos, word) in words.iter() {
            println!("{} {}", pos, word);
        }

        assert_eq!(words.len(), 24);
        let expected = vec![
            (0_usize, "aF"),
            (0, "aH"),
            (0, "aL"),
            (0, "baR"),
            (0, "bI"),
            (0, "bO"),
            (1, "aF"),
            (1, "aH"),
            (1, "aL"),
            (1, "bI"),
            (1, "bO"),
            (6, "aF"),
            (6, "aH"),
            (6, "aL"),
            (6, "baR"),
            (6, "bI"),
            (6, "bO"),
            (7, "aF"),
            (7, "aH"),
            (7, "aL"),
            (7, "bI"),
            (7, "bO"),
            (13, "af"),
            (13, "Af"),
        ];
        let expected = expected.iter().collect::<HashSet<_>>();

        for (i, w) in words.into_iter() {
            let s = w.to_string();
            let m = (i, s.as_str());
            assert!(expected.contains(&m));
        }
        Ok(())
    }

    #[test]
    fn test_get_legal_characters() -> Result<()> {
        let wordlist = Wordlist::from_words(WORDS, &Codec::default()).unwrap();
        let word = Row::try_from(" ")?;
        let lc = wordlist.get_legal_characters(&word);
        assert_eq!(lc.to_string(), "abefhilorst");
        Ok(())
    }
}
