use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Flag<L> {
    Short(char),
    Long(L),
}

impl<L> From<char> for Flag<L> {
    fn from(short: char) -> Self {
        Flag::Short(short)
    }
}

impl<'a> From<&'a str> for Flag<&'a str> {
    fn from(long: &'a str) -> Self {
        Flag::Long(long)
    }
}

impl From<String> for Flag<String> {
    fn from(long: String) -> Self {
        Flag::Long(long)
    }
}

impl<'b, L: fmt::Display> fmt::Display for Flag<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Flag::Short(short)   => write!(f, "-{}", short),
            Flag::Long(ref long) => write!(f, "--{}", long),
        }
    }
}
