use anyhow::Result;
use wordfeud_solver::{Board, Language, List};

const TEST_STATE: &[&str] = &[
    "...............",
    "...............",
    "............z..",
    "............if.",
    ".........dental",
    "..........v.ex.",
    ".......h..e....",
    "......hedonIc..",
    "....r..d..l....",
    "....o..o..y....",
    "....brent......",
    "....o..i..v....",
    ".gaits.S..e....",
    "....i..munged..",
    "....c.....a....",
];

fn run() -> Result<()> {
    let board = Board::new(Language::EN)
        .with_state_from_strings(&TEST_STATE)?
        .with_wordlist_from_file("../wordlists/wordlist-en-sowpods.txt")?;
    let letters = board.encode("koetsje")?;
    board.calc_all_word_scores(letters)?;
    let maxdist: usize = 7;
    let wordlist = board.wordlist();
    let mut state_str: Vec<Vec<char>> = TEST_STATE
        .iter()
        .map(|&row| row.chars().collect())
        .collect();
    for (i, (&row, rowdata)) in board
        .horizontal()
        .iter()
        .zip(board.rowdata(true))
        .enumerate()
    {
        let indices = wordlist.start_indices(row, rowdata, maxdist);
        // println!("{} {:?}", i, indices);

        for j in 0..row.len() {
            if state_str[i][j] == '.' && indices.contains(&j) {
                state_str[i][j] = '+'
            }
        }
    }
    println!("Start positions (indicated by '+'):");
    for row in state_str {
        println!("{}", row.into_iter().collect::<String>());
    }

    let mut state_str: Vec<Vec<char>> = TEST_STATE
        .iter()
        .map(|&row| row.chars().collect())
        .collect();
    for (i, (&row, rowdata)) in board
        .horizontal()
        .iter()
        .zip(board.rowdata(true))
        .enumerate()
    {
        for j in 0..row.len() {
            if state_str[i][j] == '.' && rowdata[j].1 {
                state_str[i][j] = '*'
            }
        }
    }
    println!("Connected positions (indicated by '*'):");
    for row in state_str {
        println!("{}", row.into_iter().collect::<String>());
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {:?}", err);
    }
}
