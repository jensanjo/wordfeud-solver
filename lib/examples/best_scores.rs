use anyhow::Result;
use std::time::Instant;
use wordfeud_solver::{find_best_scores, Board, Language};

const STATE: &[&str] = &[
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

fn run() -> Result<()> {
    eprintln!("find_best_score");
    let wordfile = "../wordlists/wordlist-nl.txt";
    // let now = Instant::now();
    let mut board = Board::new(Language::NL)
        .with_wordlist_from_file(wordfile)?
        .with_state_from_strings(STATE)?;
    let rack = board.encode("bmekqev")?;
    let nsamples = 50;
    let now = Instant::now();
    let mut scores = find_best_scores(&mut board, rack, nsamples)?;
    let dt = now.elapsed().as_secs_f32();
    println!("get scores took {:.2} s", dt);
    scores.sort_by(|a, b| b.adj_score.cmp(&a.adj_score));
    // scores.sort_by(|a, b|, b.6.cmp(&a.6));
    for s in scores.into_iter().take(10) {
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
