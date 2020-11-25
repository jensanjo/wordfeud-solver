use anyhow::Result;
use std::convert::TryFrom;
use std::time::Instant;
use wordfeud_solver::{Board, Letters};

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

fn main() -> Result<()> {
    let t0 = Instant::now();
    #[cfg(feature = "bincode")]
    let board = Board::new("nl")
        .wordlist_deserialize_from("wordlists/wordlist-nl.bin")
        .unwrap();
    #[cfg(not(feature = "bincode"))]
    let board = Board::new("nl")
        .wordlist_from_file("wordlists/wordlist-nl.txt")
        .unwrap();
    let board = board.state_from_strings(&TEST_STATE);
    let dt = t0.elapsed();
    println!("Create board with wordlist took {:?}", dt);
    let letters = Letters::try_from("koetsje")?;
    board.calc_all_word_scores(letters);

    for &letters in &["koetsje", "mdjenj*"] {
        let t0 = Instant::now();
        let letters = Letters::try_from(letters)?;
        let mut results = board.calc_all_word_scores(letters);
        let dt = t0.elapsed();
        println!(
            "Calc all word scores with {}: {} results in {:?}",
            letters,
            results.len(),
            dt
        );
        // find the best 20 results
        results.sort_by(|a, b| (b.4).cmp(&a.4));
        for &(x, y, hor, word, score) in results.iter().take(20) {
            println!("({}, {}, {}, \"{}\", {}", x, y, hor, word, score);
        }
    }
    Ok(())
}
