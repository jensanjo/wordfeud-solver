use anyhow::Result;
use std::time::Instant;
use wordfeud_solver::{find_best_scores, remaining_tiles, Board, TileBag, Language};

const STATE: &[&str] = &[
    "..............d",
    "..............r",
    ".........v....a",
    "........co.gauw",
    ".......pol.r...",
    "......fa.snIb..",
    ".....kapo..j...",
    "....zakten.s...",
    "...wel.ere..t.y",
    "..heek..si.me.e",
    "client..a.mexen",
    "...t.e..a...e..",
    ".bijen.gierend.",
    ".u.eh.does...u.",
    ".s.....dr..doft",    
];

const GRID: &[&str] = &[
    "-- -- 2l -- -- -- 2w -- 2w 2w -- -- -- 2l --",
    "-- 2w -- -- 3l -- 2l -- -- -- -- 2l -- 2l --",
    "-- -- 2l -- 3l -- -- -- -- -- -- -- -- -- --",
    "-- -- -- -- -- -- -- -- -- -- -- -- -- -- --",
    "-- -- -- -- -- -- -- -- -- -- 3l -- -- -- --",
    "-- 3w 2l -- -- -- -- -- -- -- -- -- -- -- --",
    "-- 2w -- -- 3l -- -- -- -- 3l -- -- -- -- --",
    "-- 3l -- -- -- -- -- ss -- -- -- -- -- -- 2w",
    "-- -- -- -- -- -- -- -- -- -- -- 3l -- -- --",
    "3l -- -- -- -- -- -- -- -- -- -- -- -- -- --",
    "-- -- -- -- -- -- -- -- -- -- -- -- -- -- --",
    "3w 3l -- -- -- -- -- -- -- -- -- -- -- -- 2w",
    "-- -- -- -- -- -- -- -- -- -- -- -- -- -- 2l",
    "-- -- -- -- -- -- -- -- -- -- -- -- -- -- --",
    "-- -- 2l -- 3l -- -- -- -- -- -- -- -- -- --",      
];

fn run() -> Result<()> {
    eprintln!("find_best_score");
    let wordfile = "../wordlists/wordlist-nl.txt";
    let mut board = Board::new(Language::NL)
        .with_wordlist_from_file(wordfile)?
        .with_grid_from_strings(GRID)?
        .with_state_from_strings(STATE)?;

    let rack = board.encode("gnnnoqz")?;
    // let rack = board.encode("mnnv*")?;
    let full_bag = TileBag::from(board.tileset());
    let remaining = remaining_tiles(&full_bag, &board, rack);
    eprintln!("Remaining tiles: {:?}", remaining);
    let nsamples = 50;
    let now = Instant::now();
    let mut scores = find_best_scores(&mut board, rack, nsamples)?;
    let dt = now.elapsed().as_secs_f32();
    eprintln!("get scores took {:.2} s", dt);

    scores.sort_by_key(|item| std::cmp::Reverse(item.adj_score));

    for s in scores.into_iter().take(20) {
        println!(
            "{:2} {:2} {:1} {:-7} {:3} {:4} {:-7}",
            s.x,
            s.y,
            s.horizontal as i32,
            s.word.to_uppercase(),
            s.score,
            s.adj_score,
            s.played,
        );
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {:?}", err);
    }
}
