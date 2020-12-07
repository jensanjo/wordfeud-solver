use std::io::Result;
use wordfeud_solver::{Codec, Wordlist};

#[cfg(feature = "serde")]
fn serialize_wordlist(wordlist: Wordlist) -> Result<()> {
    use std::fs::File;
    use std::io::prelude::*;
    // save to bin file
    let serialized = bincode::serialize(&wordlist).unwrap();
    let mut file = File::create("../wordlists/wordlist-nl.bin")?;
    file.write_all(&serialized)?;
    Ok(())
}

fn main() -> Result<()> {
    let wordfile = "../wordlists/wordlist-nl.txt";
    let wordlist = Wordlist::from_file(wordfile, &Codec::default()).unwrap();
    println!("{}", wordlist);
    #[cfg(feature = "serde")]
    serialize_wordlist(wordlist)?;
    Ok(())
}
