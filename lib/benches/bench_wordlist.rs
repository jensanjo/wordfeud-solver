use criterion::{criterion_group, criterion_main, Criterion};
use wordfeud_solver::{Codec, RowData, Wordlist};

const WORDS: &[&str] = &[
    "af", "ah", "al", "aar", "aas", "bi", "bo", "bar", "bes", "bel",
];
const WORDFILE: &str = "wordlists/wordlist-nl.txt";
const WORDFILE_SMALL: &str = "wordlists/words.txt";

fn bench_from_words() {
    let _wordlist = Wordlist::from_words(WORDS, &Codec::default());
}

fn bench_from_file() {
    let _wordlist = Wordlist::from_file(WORDFILE_SMALL, &Codec::default()).unwrap();
}

fn bench_words(
    c: &mut Criterion,
    name: &str,
    wordlist: Wordlist,
    row: &str,
    rowdata: &RowData,
    letters: &str,
) {
    let row = wordlist.encode(row).unwrap();
    let letters = wordlist.encode(letters).unwrap();
    c.bench_function(name, |b| {
        b.iter(|| {
            wordlist
                .words(&row, &rowdata, &letters, None)
                .collect::<Vec<_>>()
        })
    });
}

fn bench_get_legal_characters(c: &mut Criterion) {
    let wordlist = Wordlist::from_words(WORDS, &Codec::default()).unwrap();
    let word = wordlist.encode("a ").unwrap();
    c.bench_function("wordlist.get_legal_characters", |b| {
        b.iter(|| wordlist.get_legal_characters(&word))
    });
}

fn bench_node_matches(c: &mut Criterion, name: &str, wordfile: &str, letters: &str) {
    //let wordlist = Wordlist::from_words(WORDS);
    let wordlist = Wordlist::from_file(wordfile, &Codec::default()).unwrap();
    let row = wordlist.encode("    ").unwrap();
    let rowdata: RowData = wordlist.connected_row(&row);
    let pos = 0;
    let letters = wordlist.encode(letters).unwrap();
    c.bench_function(name, |b| {
        b.iter(|| {
            wordlist
                .matches(0, row, &rowdata, pos, &letters)
                .collect::<Vec<_>>()
        })
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("wordlist.from_words", |b| b.iter(bench_from_words));
    c.bench_function("wordlist.from_file", |b| b.iter(bench_from_file));

    let wordlist = Wordlist::from_file(WORDFILE, &Codec::default()).unwrap();
    let labels = wordlist.all_labels;
    let rowdata = RowData::from_array_len([(labels, true); 16], 15);
    let row = "    t     c   f";
    let letters = "polkas*";
    let wordlist = Wordlist::from_file(WORDFILE, &Codec::default()).unwrap();
    bench_words(c, "wordlist.words.1", wordlist, row, &rowdata, letters);
    bench_get_legal_characters(c);
    bench_node_matches(c, "node.matches.1", WORDFILE, "abel");
    bench_node_matches(c, "node.matches.2", WORDFILE, "mdjen*");
    bench_node_matches(c, "node.matches.3", WORDFILE, "mdjen**");
    bench_node_matches(c, "node.matches.4", WORDFILE, "vakjet*");
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
