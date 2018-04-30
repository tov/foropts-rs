# foropts-rs: iterator-style option parsing for Rust

[![Build Status](https://travis-ci.org/tov/foropts-rs.svg?branch=master)](https://travis-ci.org/tov/foropts-rs)
[![Crates.io](https://img.shields.io/crates/v/foropts.svg?maxAge=2592000)](https://crates.io/crates/foropts)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](LICENSE-APACHE)

Most argument parsing libraries, such as [`clap`](https://clap.rs/) treat the arguments as a
multimap; `foropts` treats the arguments as a sequence. This usually isn’t what you want, but
occasionally it is.

# Usage

It's on [crates.io](https://crates.io/crates/foropts), so you can add

```toml
foropts = "0.1"
```

to your `Cargo.toml` and

```rust
extern crate foropts;
```

to your crate root.

# Example

In this example, we accept one boolean flag, `-v` (or `--verbose`), and two
string options, `-b` (or `--before`) and `-a` (or `--after`). The string options
build a string, where the relative order of the appearances of `-a` and `-b` matters.
This is hard to do when your arguments are treated as a multimap, but easy when
you can iterate over them sequentially.

```
# use foropts;
enum Opt {
    Before(String),
    After(String),
    Verbose,
}

let config =
    foropts::Config::new("build_string_example")
        .arg(foropts::Arg::parsed_param("BEFORE", Opt::Before)
             .short('b').long("before"))
        .arg(foropts::Arg::parsed_param("AFTER", Opt::After)
             .short('a').long("after"))
        .arg(foropts::Arg::flag(|| Opt::Verbose)
             .short('v').long("verbose"));

let mut verbose     = false;
let mut accumulator = String::new();

let opts = ["-b1", "-va", "2", "--after=3", "--before", "4"]
    .iter().map(ToString::to_string);

for opt in config.iter(opts) {
    match opt.unwrap() {
        Opt::Before(s) => accumulator = s + &accumulator,
        Opt::After(s)  => accumulator = accumulator + &s,
        Opt::Verbose   => verbose = true,
    }
}

assert_eq!( "4123", accumulator );
assert!( verbose );
```

