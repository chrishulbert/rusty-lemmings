# Rusty Lemmings

![Lemmings](https://raw.githubusercontent.com/chrishulbert/rusty-lemmings/master/github/logo.png)

Rewrite of the old game Lemmings in Rust

You'll need to make a '~/Lemmings' folder and unzip the lemmings variants there. Eg:

    ~/Lemmings/lemmings/main.dat
    ~/Lemmings/ohnomore/*.dat
    ~/Lemmings/christmas1991/*.dat
    ~/Lemmings/christmas1992/*.dat
    ~/Lemmings/holiday1993/*.dat
    ~/Lemmings/holiday1994/*.dat
 
Filenames will need to be lowercase, which can be fixed like so: `ruby -e "Dir['*'].each { |p| File.rename(p, p.downcase) }"`

You can find lemmings here: https://www.camanis.net/lemmings/lemmings.php

Install rust (eg `brew install rustup` then `rustup update`), then do `cargo run`.

## Compilation notes

To cross compile, first run `rustup target list` to see supported platforms.
If you are on apple silicon macOS, run `rustup target add x86_64-apple-darwin` to support Intel macs.
Note: Run this from the project's folder so it applies the toolchain version.
Then use cargo specifying the target as follows: `cargo build --target=x86_64-apple-darwin`
