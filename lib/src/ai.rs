#![allow(dead_code, unused_variables, unused_mut, unused_assignments)]
use crate::tilebag::TileBag;
use crate::tiles::BLANK;
use crate::{Board, Code, Error, Item, Letter, Letters, TileSet};
use rand::{rngs::StdRng, seq::IteratorRandom, Rng, SeedableRng};
use std::convert::{From, TryFrom};

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, Copy)]
pub enum ExitFlag {
    None = 0,
    Our = 1,
    Opponent = 2,
}
/// Returned score information. Extended from [board::Score](crate::Score)
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone)]
pub struct Score {
    /// word start x: 0..N
    pub x: usize,
    /// word start y: 0..N
    pub y: usize,
    /// horizontal if true, else vertical
    pub horizontal: bool,
    /// word as String
    pub word: String,
    /// score for this word
    pub score: i32,
    /// score adjusted for opponent score
    pub adj_score: i32,
    /// played letters
    pub played: String,
    /// exit flag:
    pub exit_flag: ExitFlag,
    /// std deviation of best opponent scores
    pub std: f32,
}

pub fn used_tiles(board: &Board, rack: Letters) -> TileBag {
    let mut used_tiles: Vec<_> = board
        .horizontal()
        .iter()
        .map(|&row| row.into_iter())
        .flatten()
        .filter_map(|cell| cell.tile())
        .map(|tile| {
            if tile.is_wildcard() {
                BLANK
            } else {
                tile.code()
            }
        })
        .collect();
    used_tiles.extend(rack.into_iter().map(|letter: Letter| letter.code()));
    TileBag::from(&used_tiles)
}

fn remaining_tiles(full_bag: &TileBag, board: &Board, rack: Letters) -> TileBag {
    full_bag.clone() - used_tiles(board, rack)
}

fn tiles_score(tiles: &Letters, tileset: &TileSet) -> i32 {
    let score: u32 = tiles
        .into_iter()
        .map(|letter| tileset.points(letter.code()))
        .sum();
    score as i32
}

fn draw_random_tiles<R: Rng>(tiles: &[Code], n: usize, mut rng: &mut R) -> Vec<Code> {
    tiles
        .iter()
        .choose_multiple(&mut rng, n)
        .into_iter()
        .cloned()
        .collect()
}

/// Find the best moves on a wordfeud board, considering opponent moves.
///
/// First, calculate the scores with our letters with [calc_all_word_scores](Board::calc_all_word_scores).
/// Then we adjust the scores by evaluating possible opponent moves with the remaining letters, as explained below.
///
/// ## MIDGAME: tiles left in bag
///
/// For each of the 20 best words:
/// - Play the word on the board.
/// - For each of `nsamples` randow draws of 7 tiles from the remaining tiles:
///     - Calculate all word scores for the opponent.
///     - Add the best score to a list of opponent tiles score
/// - Calculate the mean opponent score. Subtract it from our score
///
/// ## ENDGAME: no tiles left in bag
///
/// For each of our words:
/// - Play the word on the board.
///     - CASE 1:
///         - If we can exit: value = our score + opponents tile score
///     - CASE 2:
///         - Calculate all word scores for the opponent.
///         - If opponent can exit: value = our word - (opponents word + our tiles score)
///     - CASE 3:
///         - No one can exit. value = our word - oppenents max score
///
/// # Errors
/// Returns [Error](crate::Error) when:
/// * calculate scores from our letters fails (see [calc_all_word_scores](crate::Board::calc_all_word_scores))
/// * one of our words can not be played on the board (see [play_word](crate::Board::play_word))
/// * sample opponent scores fails (see [sample_scores](crate::Board::sample_scores))
pub fn find_best_scores(
    board: &mut Board,
    rack: Letters,
    nsamples: usize,
) -> Result<Vec<Score>, Error> {
    let mut result = Vec::new();
    let mut rng = StdRng::seed_from_u64(123); // seeded to get reproducible results.
    let full_bag = TileBag::from(board.tileset());
    let remaining = remaining_tiles(&full_bag, board, rack);
    let mut tiles: Vec<u8> = remaining.iter().cloned().collect(); // remaining tiles as Vec<Code>
    tiles.sort_unstable();

    // calculate word scores for our letters
    let mut words = board.calc_all_word_scores(rack)?;
    if words.is_empty() {
        return Ok(result);
    }
    words.sort_by(|a, b| b.score.cmp(&a.score));

    let mut opp_tiles_score: i32 = 0;
    let in_endgame = remaining.len() < 7;

    // In endgame the opponent letters are known, calculate all possible opponent moves.
    // Otherwise, prepare a bunch of random samples from remaining letters and calculate best opponent moves with each
    let samples: Vec<Letters>;
    let top_n; // number of our best scores to evaluate: all in endgame, otherwise 20
    if in_endgame {
        // one sample
        let sample = Letters::try_from(tiles)?; // remaining tiles
        samples = vec![sample];
        opp_tiles_score = tiles_score(&rack, board.tileset());
        top_n = words.len(); // evaluate all our possible words in endgame
    } else {
        // random samples from remaining letters
        samples = (0..nsamples)
            .into_iter()
            .map(|_| {
                let v = draw_random_tiles(&tiles, 7, &mut rng);
                Letters::try_from(v).unwrap()
            })
            .collect();
        top_n = 20; // evaluate up to 20 of our best words
    }
    let saved_state = board.horizontal();
    for (i, &s) in words.iter().take(top_n).enumerate() {
        let letters = board.decode(s.word);
        let played = board.play_word(&letters, s.x, s.y, s.horizontal, true)?;
        let mut exit_flag = ExitFlag::None;
        let mut opp_scores = Vec::new();
        if in_endgame && played.len() == letters.len() {
            // TODO CHECK if this works for non-ascii or multichar tiles
            exit_flag = ExitFlag::Our;
            opp_scores.push(-opp_tiles_score);
        } else {
            let res = board.sample_scores(&samples, s.score, false)?;
            opp_scores = res.iter().map(|&(score, _)| score as i32).collect();
            if res.iter().any(|&(_, opp_exit)| opp_exit) {
                exit_flag = ExitFlag::Opponent;
            }
        }

        board.set_state(&saved_state); // restore board before next iteration

        let mean_opp_score = mean(&opp_scores).unwrap_or(0.0);
        let std_opp_score = std_deviation(&opp_scores).unwrap_or(0.0);
        let adj_opp_score = s.score as f32 - mean_opp_score;
        let res = Score {
            x: s.x,
            y: s.y,
            horizontal: s.horizontal,
            word: board.decode(s.word),
            score: s.score as i32,
            adj_score: adj_opp_score.round() as i32,
            played,
            exit_flag,
            std: std_opp_score,
        };
        result.push(res);
    }

    Ok(result)
}

/// From [rust cookbook](https://rust-lang-nursery.github.io/rust-cookbook/science/mathematics/statistics.html)
fn mean(data: &[i32]) -> Option<f32> {
    let sum = data.iter().sum::<i32>() as f32;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}

/// From [rust cookbook](https://rust-lang-nursery.github.io/rust-cookbook/science/mathematics/statistics.html)
fn std_deviation(data: &[i32]) -> Option<f32> {
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = data
                .iter()
                .map(|value| {
                    let diff = data_mean - (*value as f32);

                    diff * diff
                })
                .sum::<f32>()
                / count as f32;

            Some(variance.sqrt())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::Board;
    use anyhow::Result;

    use super::*;
    use crate::Language;

    const TEST_STATE: &[&str] = &[
        "........gezoend",
        ".........xi...i",
        ".....grif.jap.e",
        ".......dauw...v",
        "....her...I...E",
        ".....rennen.e.n",
        "..........d.i..",
        ".......hesjes..",
        "......yen.e.e..",
        ".....bof.......",
        "kolkte.turns...",
        ".......e.......",
        ".......n.......",
        "...............",
        "...............",
    ];

    #[test]
    fn test_remaining_letters() -> Result<()> {
        let board = Board::new(Language::NL)
            .with_state_from_strings(TEST_STATE)
            .unwrap();
        let bag = TileBag::from(board.tileset());
        let rack = "mndceng";
        let rack_tiles: Letters = board.encode(rack)?;
        println!(
            "{} {:?} {}",
            rack,
            rack_tiles,
            tiles_score(&rack_tiles, board.tileset())
        );
        let used = used_tiles(&board, rack_tiles);
        println!("full bag: {} used: {}", bag.len(), used.len());
        let remaining = remaining_tiles(&bag, &board, rack_tiles);
        println!("remaining: {} {:?}", remaining.len(), remaining);
        for (code, (tag, _, _)) in board.tileset().tiles.iter().enumerate() {
            let code = code as u8;
            println!(
                "{:2} {:2} {:2} {:2} {:2}",
                code,
                tag,
                bag.count_of(&code),
                used.count_of(&code),
                remaining.count_of(&code)
            );
            assert_eq!(
                bag.count_of(&code),
                used.count_of(&code) + remaining.count_of(&code)
            );
        }
        Ok(())
    }
}
