use anyhow::Result;
use std::time::Instant;
use wordfeud_solver::{Board, Language};

const TEST_STATE: &[&str] = &[
    "    t     c   f",
    "    e    he   o",
    "    r   bis g k",
    "    u  bol te v",
    "    gepof dimme",
    "      la vree e",
    "    qua   ene  ",
    "      Spoelen  ",
    "     s a   n   ",
    "     c d we    ",
    "     hadden    ",
    "    nu o   y   ",
    "  wrat siJzen  ",
    "    k     os   ",
    "   zerk   g    ",
];

fn run() -> Result<()> {
    let t0 = Instant::now();
    #[cfg(feature = "bincode")]
    let board =
        Board::new(Language::NL).with_wordlist_deserialize_from("../wordlists/wordlist-nl.bin")?;
    #[cfg(not(feature = "bincode"))]
    let board = Board::new(Language::NL).with_wordlist_from_file("../wordlists/wordlist-nl.txt")?;
    let board = board.with_state_from_strings(&TEST_STATE)?;
    let dt = t0.elapsed();
    println!("Create board with wordlist took {:?}", dt);
    let letters = "koetsje";
    board.calc_all_word_scores(letters)?;

    for &letters in &["koetsje", "mdjenj*"] {
        let t0 = Instant::now();
        let mut results = board.calc_all_word_scores(letters)?;
        let dt = t0.elapsed();
        println!(
            "Calc all word scores with {}: {} results in {:?}",
            letters,
            results.len(),
            dt
        );
        // find the best 20 results
        results.sort_by(|a, b| (b.score).cmp(&a.score));
        for s in results.into_iter().take(20) {
            println!(
                "({}, {}, {}, \"{}\", {}",
                s.x,
                s.y,
                s.horizontal,
                board.decode(s.word),
                s.score
            );
        }
    }
    // Play a word
    let mut board = board;
    board.play_word("jokert", 0, 2, true, true)?;
    println!("{}", board);
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {:?}", err);
    }
}
