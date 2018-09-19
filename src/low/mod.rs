//! A low-level, borrowing argument parser.

mod config;
mod flag;
mod iter;

pub use self::config::{Config, HashConfig, Presence};
pub use self::flag::Flag;
pub use self::iter::{Iter, Item, ErrorKind};

#[cfg(test)]
mod tests {
    use super::*;

    fn _long_short_owned() -> HashConfig<String> {
        HashConfig::new()
            .short('a', false)
            .long("all", false)
            .short('o', true)
            .long("output", true)
    }

    fn _long_short_ref() -> HashConfig<&'static str> {
        HashConfig::new()
            .short('a', false)
            .long("all", false)
            .short('o', true)
            .long("output", true)
    }

    #[test]
    fn owned() {
        let config = HashConfig::new()
            .opt('a', false)
            .opt("all".to_owned(), false)
            .opt('o', true)
            .opt("output".to_owned(), true);

        let result: Vec<_> = config.parse_slice(&["-a", "-ofile"]).collect();

        assert_eq!( result,
                    &[ Item::Opt(Flag::Short('a'), None),
                       Item::Opt(Flag::Short('o'), Some("file")) ] );
    }

    #[test]
    fn borrowed() {
        assert_parse( &["-a", "-ofile"],
                      &[ Item::Opt(Flag::Short('a'), None),
                         Item::Opt(Flag::Short('o'), Some("file")) ] );
    }

    fn assert_parse(input: &[&str], output: &[Item]) {
        let config = HashConfig::new()
            .opt('a', false)
            .opt("all", false)
            .opt('o', true)
            .opt("output", true);

        let result: Vec<_> = config.parse_slice(input).collect();
        assert_eq!( result, output );
    }
}

