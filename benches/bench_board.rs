use criterion::{criterion_group, criterion_main, Criterion};
use std::convert::TryFrom;
use wordfeud_solver::{Board, Language, Letters};

const WORDFILE: &str = "../wordfeud/wordlists/wordlist-nl.txt";
// TODO use different (generated) boards
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

fn bench_calc_all_word_scores(c: &mut Criterion, name: &str, letters: &str) {
    let board = Board::new(Language::NL)
        .with_wordlist_from_file(WORDFILE)
        .unwrap()
        .with_state_from_strings(&TEST_STATE);

    let letters = Letters::try_from(letters).unwrap();
    c.bench_function(&format!("board.{}", name), |b| {
        b.iter(|| {
            let mut results = board.calc_all_word_scores(letters);
            results.sort_by(|a, b| (b.4).cmp(&a.4));
        })
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_calc_all_word_scores(c, "1", "abel");
}

fn slow_benchmarks(c: &mut Criterion) {
    bench_calc_all_word_scores(c, "2", "mdjenj*");
    bench_calc_all_word_scores(c, "3", "polkas*");
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(60);
    targets = criterion_benchmark
}

criterion_group! {
    name = slow;
    config = Criterion::default()
        .sample_size(10);
    targets = slow_benchmarks
}

criterion_main!(benches, slow);
