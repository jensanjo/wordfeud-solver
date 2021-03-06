# Wordfeud Solver
[![Crates.io](https://img.shields.io/crates/v/wordfeud-solver)](https://crates.io/crates/wordfeud-solver)
[![Documentation](https://docs.rs/wordfeud-solver/badge.svg)](https://docs.rs/wordfeud-solver)

A Rust library to evaluate all possible moves given a wordfeud board and rack.
The wordfeud solver needs a wordlist to be able to play in a given language. There are no 
wordlists included with this library. You can get a dutch wordlist [here](https://github.com/jensanjo/wordfeud-wordlists.git).

This solver is ported from the excellent [wordfeudplayer](https://github.com/mrcz/Wordfeud-Player) Python package .
By porting it to Rust it became much faster. 
