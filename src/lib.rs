#![doc(html_root_url = "https://docs.rs/foropts/0.3.6")]
//! An argument-parsing iterator.
//!
//! Most argument parsing libraries, such as
//! [`clap`](https://crates.io/crates/clap) treat the arguments as a
//! multimap; `foropts` treats the arguments as a sequence. This usually
//! isn’t what you want, but occasionally it is.
//!
//! # Usage
//!
//! It's on [crates.io](https://crates.io/crates/foropts), so you can add
//!
//! ```toml
//! [dependencies]
//! foropts = "0.3.6"
//! ```
//!
//! to your `Cargo.toml` and
//!
//! ```rust
//! extern crate foropts;
//! ```
//!
//! to your crate root.
//!
//! This crate supports Rust version 1.22 and later.
//!
//! # Example
//!
//! In this example, we accept one boolean flag, `-v` (or `--verbose`), and two
//! string options, `-b` (or `--before`) and `-a` (or `--after`). The string options
//! build a string, where the relative order of the appearances of `-a` and `-b` matters.
//! This is hard to do when your arguments are treated as a multimap, but easy when
//! you can iterate over them sequentially.
//!
//! ```
//! # use foropts;
//! enum Opt {
//!     Before(String),
//!     After(String),
//!     Verbose,
//! }
//!
//! let config =
//!     foropts::Config::new("build_string_example")
//!         .arg(foropts::Arg::parsed_param("BEFORE", Opt::Before)
//!              .short('b').long("before"))
//!         .arg(foropts::Arg::parsed_param("AFTER", Opt::After)
//!              .short('a').long("after"))
//!         .arg(foropts::Arg::flag(|| Opt::Verbose)
//!              .short('v').long("verbose"));
//!
//! let mut verbose     = false;
//! let mut accumulator = String::new();
//!
//! let opts = ["-b1", "-va", "2", "--after=3", "--before", "4"]
//!     .iter().map(ToString::to_string);
//!
//! for opt in config.iter(opts) {
//!     match opt.unwrap_or_else(|e| config.exit_error(&e)) {
//!         Opt::Before(s) => accumulator = s + &accumulator,
//!         Opt::After(s)  => accumulator = accumulator + &s,
//!         Opt::Verbose   => verbose = true,
//!     }
//! }
//!
//! assert_eq!( "4123", accumulator );
//! assert!( verbose );
//! ```

use std::str::FromStr;


    mod arg;
    mod config;
    mod error;
    mod iter;
pub mod low;
    mod util;

pub use arg::Arg;
pub use config::Config;
pub use error::{Error, Result};
pub use iter::Iter;

#[cfg(test)]
mod tests {
    use super::{Config, Arg, Result};
    use std::fmt::Debug;

    #[test]
    fn char_example() {
        let config =
            Config::new("char_example")
                .arg(Arg::flag(|| 'a').short('a'))
                .arg(Arg::flag(|| 'b').short('b'));

        let opts = ["-ab", "-ba"].iter().map(ToString::to_string);
        let result: Result<String> = config.iter(opts).collect();
        assert_eq!( Ok("abba".to_owned()), result );
    }

    #[derive(PartialEq, Debug)]
    enum FLS {
        Freq(f32),
        Louder,
        Softer,
    }

    #[test]
    fn flag_s() {
        assert_parse(&fls_config(), &["-s"], &[FLS::Softer]);
    }

    #[test]
    fn flag_s_s() {
        assert_parse(&fls_config(), &["-ss"], &[FLS::Softer, FLS::Softer]);
    }

    #[test]
    fn flag_softer() {
        assert_parse(&fls_config(), &["--softer"], &[FLS::Softer]);
    }

    #[test]
    fn flag_s_l_s() {
        let config = &fls_config();
        let expected = &[FLS::Softer, FLS::Louder, FLS::Softer];
        assert_parse(config, &["-sls"], expected);
        assert_parse(config, &["-s", "-ls"], expected);
        assert_parse(config, &["-sl", "-s"], expected);
        assert_parse(config, &["-s", "-l", "-s"], expected);
    }

    #[test]
    fn flag_f_needs_param() {
        assert_parse_error(&fls_config(), &["-f"]);
    }

    #[test]
    fn flag_freq_needs_param() {
        assert_parse_error(&fls_config(), &["--freq"]);
    }

    #[test]
    fn flag_freq_needs_float_param() {
        let config = &fls_config();
        assert_parse_error(config, &["-fhello"]);
        assert_parse_error(config, &["-f", "hello"]);
        assert_parse_error(config, &["--freq=hello"]);
        assert_parse_error(config, &["--freq", "hello"]);

        assert_parse(config, &["-f5.5"], &[FLS::Freq(5.5)]);
        assert_parse(config, &["-f", "5.5"], &[FLS::Freq(5.5)]);
        assert_parse(config, &["--freq=5.5"], &[FLS::Freq(5.5)]);
        assert_parse(config, &["--freq", "5.5"], &[FLS::Freq(5.5)]);
    }

    #[test]
    fn float_parsing_error_message() {
        assert_parse_error_matches(&fls_config(), &["-fhello"],
                                   "option -fhello: invalid float literal");
    }

    fn fls_config() -> Config<'static, FLS> {
        Config::new("fls")
            .arg(Arg::parsed_param("FREQ", FLS::Freq).short('f').long("freq"))
            .arg(Arg::flag(|| FLS::Louder).short('l').long("louder"))
            .arg(Arg::flag(|| FLS::Softer).short('s').long("softer"))
    }

    #[derive(PartialEq, Debug)]
    enum Pos {
        FlagA,
        Positional(String),
    }

    #[test]
    fn double_hyphen_works() {
        let config = &pos_config();
        assert_parse_error(config, &["-b", "--", "-a"]);
        assert_parse(config, &["-a", "--", "-b"],
                     &[Pos::FlagA, Pos::Positional("-b".to_owned())]);
        assert_parse(config, &["-a", "--", "-a"],
                     &[Pos::FlagA, Pos::Positional("-a".to_owned())]);
        assert_parse(config, &["-aa", "bbb", "-a"],
                     &[Pos::FlagA,
                       Pos::FlagA,
                       Pos::Positional("bbb".to_owned()),
                       Pos::FlagA]);
    }

    #[test]
    fn unrecognized_option_works() {
        assert_parse_error_matches(&pos_config(),
                                   &["-b"],
                                   "option -b: unrecognized");
    }

    fn pos_config() -> Config<'static, Pos> {
        Config::new("pos")
            .arg(Arg::flag(|| Pos::FlagA).short('a'))
            .arg(Arg::parsed_param("POS", Pos::Positional))
    }

    fn assert_parse_error_matches<T>(config: &Config<T>, args: &[&str], pattern: &str) {
        match parse(config, args) {
            Ok(_)  => panic!("expected parse failure, got success"),
            Err(e) => assert!( e.to_string().matches(pattern).next().is_some(),
                               format!("{:?} does not match {:?}", e.to_string(), pattern)),
        }
    }

    fn assert_parse_error<T>(config: &Config<T>, args: &[&str]) {
        assert!( parse(config, args).is_err() );
    }

    fn assert_parse<T>(config: &Config<T>, args: &[&str], expected: &[T])
        where T: Debug + PartialEq
    {
        assert_eq!( parse(config, args).as_ref().map(Vec::as_slice),
                    Ok(expected) );
    }

    fn parse<T>(config: &Config<T>, args: &[&str]) -> Result<Vec<T>> {
        let args = args.into_iter().map(ToString::to_string);
        config.iter(args).collect()
    }
}
