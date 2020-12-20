#![allow(dead_code, unused_variables, unused_mut, unused_assignments)]
use crate::tilebag::TileBag;
use crate::tiles::BLANK;
use crate::{Board, Error, Item, Letter, Letters, TileSet};
use rand::{rngs::StdRng, seq::IteratorRandom, SeedableRng};
use std::convert::TryFrom;

#[derive(Debug)]
pub enum ExitFlag {
    None,
    Our,
    Opponent,
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
    TileBag::from_tiles(&used_tiles)
}

fn get_remaining_tiles(full_bag: &TileBag, board: &Board, rack: Letters) -> TileBag {
    full_bag.clone() - used_tiles(board, rack)
}

fn tiles_score(tiles: &Letters, tileset: &TileSet) -> i32 {
    let score: u32 = tiles
        .into_iter()
        .map(|letter| tileset.points(letter.code()))
        .sum();
    score as i32
}

/// Returned score information: x, y, horizontal, word, score, adjusted score, played letters, exitflag, opp score std
pub type Score = (
    usize,
    usize,
    bool,
    String,
    i32,
    i32,
    String,
    ExitFlag,
    f32,
); // TODO: simplify, put in struct?

pub fn find_best_score(
    board: &mut Board,
    rack: Letters,
    nsamples: usize,
) -> Result<Vec<Score>, Error> {
    let mut result = Vec::new();
    let mut rng = StdRng::seed_from_u64(123); // seeded to get reproducible results. TODO make global?
                                              // let mut rng =  thread_rng();
    let full_bag = TileBag::from_tileset(board.tileset());
    let remaining = get_remaining_tiles(&full_bag, board, rack);
    let mut tiles: Vec<u8> = remaining.iter().cloned().collect(); // remaining tiles as Vec<Code>
    tiles.sort_unstable();
    // let sample = board.decode(Letters::try_from(tiles)?); // remaining tiles as string
    // eprintln!("remaining tiles \"{:?}\"", tiles);

    let letters = board.decode(rack);
    eprintln!(
        "Find best score with \"{}\", {} letters left",
        letters,
        remaining.len()
    );

    // calculate word scores for our letters
    let mut words = board.calc_all_word_scores(&letters)?;
    if words.is_empty() {
        eprintln!("No words found");
        return Ok(result);
    }
    words.sort_by(|a, b| b.4.cmp(&a.4));
    let mut topn = &words[..];

    let mut opp_tiles_score: i32 = 0;
    let in_endgame = remaining.len() < 7;

    // In endgame the opponent letters are known, calculate all possible opponent moves.
    // Otherwise, prepare a bunch of random samples from remaining letters and calculate best opponent moves with each
    let samples: Vec<String>;
    if in_endgame {
        // one sample
        let sample = board.decode(Letters::try_from(tiles)?); // remaining tiles as string
        samples = vec![sample];
        opp_tiles_score = tiles_score(&rack, board.tileset());
        println!("In endgame, found {} words", words.len());
    } else {
        // random samples from remaining letters
        samples = (0..nsamples)
            .into_iter()
            .map(|_| {
                let v: Vec<u8> = tiles
                    .iter()
                    .choose_multiple(&mut rng, 7)
                    .into_iter()
                    .cloned()
                    .collect();
                let letters = Letters::try_from(v).unwrap();
                board.decode(letters)
            })
            .collect();
        topn = &words[0..20];
    }
    // println!("samples: {:?}", samples);
    let saved_state = board.horizontal();
    for (i, &(x, y, horizontal, word, score)) in topn.iter().enumerate() {
        let letters = board.decode(word);
        // eprintln!("{} {}", i, letters);
        let played = board.play_word(&letters, x, y, horizontal, true)?;
        let mut exit_flag = ExitFlag::None;
        let mut opp_scores = Vec::new();
        if in_endgame && played.len() == letters.len() {
            // TODO CHECK if this works for non-ascii or multichar tiles
            exit_flag = ExitFlag::Our;
            opp_scores.push(-opp_tiles_score);
        } else {
            let res = board.sample_scores(&samples, score, false)?;
            opp_scores = res.iter().map(|&(score, _)| score as i32).collect();
            if res.iter().any(|&(_, opp_exit)| opp_exit) {
                exit_flag = ExitFlag::Opponent;
            }
        }

        board.set_state(&saved_state); // restore board TODO we could save the complete board state, avoid recalculation

        let mean_opp_score = mean(&opp_scores).unwrap_or(0.0);
        let std_opp_score = std_deviation(&opp_scores).unwrap_or(0.0);
        let adj_opp_score = score as f32 - mean_opp_score;
        let res = (
            x,
            y,
            horizontal,
            board.decode(word),
            score as i32,
            adj_opp_score.round() as i32,
            played,
            exit_flag,
            std_opp_score,
        );
        result.push(res);
    }

    Ok(result)
}

fn mean(data: &[i32]) -> Option<f32> {
    let sum = data.iter().sum::<i32>() as f32;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}

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
    // use std::collections::HashMap;
    use crate::Board;
    use anyhow::Result;
    use std::time::Instant;

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
        let bag = TileBag::from_tileset(board.tileset());
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
        let remaining = get_remaining_tiles(&bag, &board, rack_tiles);
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

    #[test]
    fn test_random_sample() -> Result<()> {
        use rand::seq::IteratorRandom;
        use std::convert::TryFrom;

        let mut rng = rand::thread_rng();
        let board = Board::new(Language::NL);
        let bag = TileBag::from_tileset(board.tileset());
        let tiles: Vec<u8> = bag.iter().cloned().collect();
        println!("{:?}", tiles);
        let samples: Vec<u8> = tiles.into_iter().choose_multiple(&mut rng, 7);
        let letters = Letters::try_from(samples)?;
        println!("{:?}", board.decode(letters));
        Ok(())
    }

    #[test]
    fn test_tiles() -> Result<()> {
        let board = Board::default();
        let tiles: Vec<u8> = vec![5, 1, 1, 9, 14, 14, 14, 18, 19, 24, 20, 20, 64, 64];
        let sample = board.decode(Letters::try_from(tiles)?);
        println!("{}", sample);
        Ok(())
    }

    #[test]
    #[ignore] // slow test, only run when requested
    fn test_find_best_score() -> Result<()> {
        eprintln!("find_best_score");
        let wordfile = "../wordlists/wordlist-nl.txt";
        // let now = Instant::now();
        let mut board = Board::new(Language::NL)
            .with_wordlist_from_file(wordfile)?
            .with_state_from_strings(TEST_STATE)?;
        let rack = board.encode("bmekqev")?;
        let nsamples = 50;
        let now = Instant::now();
        let mut scores = find_best_score(&mut board, rack, nsamples)?;
        let dt = now.elapsed().as_secs_f32();
        println!("get scores took {:.2}", dt);
        scores.sort_by(|a, b| b.6.cmp(&a.6));
        // scores.sort_by(|a, b|, b.6.cmp(&a.6));
        for (x, y, horizontal, word, score, adj_score, played, flag, opp_std) in
            scores.into_iter().take(10)
        {
            println!(
                "{:2} {:2} {:1} {:-7} {:3} {:4} {:-7}",
                x,
                y,
                horizontal as i32,
                word.to_uppercase(),
                score,
                adj_score,
                played,
            );
        }
        Ok(())
    }
}
