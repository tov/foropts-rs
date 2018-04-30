//! An argument-parsing iterator.
//!
//! Most argument parsing libraries, such as [`clap`](https://clap.rs/) treat the arguments as a
//! multimap; `foropts` treats the arguments as a sequence. This usually isn’t what you want, but
//! occasionally it is.
//!
//! # Examples
//!
//! In this example, we accept
//!
//! ```
//! # use foropts::*;
//! ```
use std::str::FromStr;

mod util;

mod arg;
mod config;
mod error;
mod iter;

pub use arg::Arg;
pub use config::Config;
pub use error::{Error, Result};
pub use iter::Iter;

#[cfg(test)]
mod tests {
    use super::{Config, Arg, Result};

    #[derive(Clone, PartialEq, Debug)]
    enum Opt {
        Louder,
        Softer,
        Freq(f32),
    }

    #[test]
    fn flag_s() {
        assert_parse(&["-s"], &[Opt::Softer]);
    }

    #[test]
    fn flag_s_s() {
        assert_parse(&["-ss"], &[Opt::Softer, Opt::Softer]);
    }

    #[test]
    fn flag_softer() {
        assert_parse(&["--softer"], &[Opt::Softer]);
    }

    #[test]
    fn flag_s_l_s() {
        let expected = &[Opt::Softer, Opt::Louder, Opt::Softer];
        assert_parse(&["-sls"], expected);
        assert_parse(&["-s", "-ls"], expected);
        assert_parse(&["-sl", "-s"], expected);
        assert_parse(&["-s", "-l", "-s"], expected);
    }

    #[test]
    fn flag_f_needs_param() {
        assert_parse_error(&["-f"]);
    }

    #[test]
    fn flag_freq_needs_param() {
        assert_parse_error(&["--freq"]);
    }

    #[test]
    fn flag_freq_needs_float_param() {
        assert_parse_error(&["-fhello"]);
        assert_parse_error(&["-f", "hello"]);
        assert_parse_error(&["--freq=hello"]);
        assert_parse_error(&["--freq", "hello"]);

        assert_parse(&["-f5.5"], &[Opt::Freq(5.5)]);
        assert_parse(&["-f", "5.5"], &[Opt::Freq(5.5)]);
        assert_parse(&["--freq=5.5"], &[Opt::Freq(5.5)]);
        assert_parse(&["--freq", "5.5"], &[Opt::Freq(5.5)]);
    }

    fn assert_parse_error(args: &[&str]) {
        assert!( parse(args).is_err() );
    }

    fn assert_parse(args: &[&str], expected: &[Opt]) {
        assert_eq!( parse(args), Ok(expected.to_owned()) );
    }

    fn parse(args: &[&str]) -> Result<Vec<Opt>> {
        let config = get_config();
        let args = args.into_iter().map(ToString::to_string);
        config.iter(args).collect()
    }

    fn get_config() -> Config<'static, Opt> {
        Config::new("moo")
            .arg(Arg::flag(|| Opt::Louder).short('l').long("louder"))
            .arg(Arg::flag(|| Opt::Softer).short('s').long("softer"))
            .arg(Arg::parsed_param("FREQ", Opt::Freq).short('f').long("freq"))
    }
}
