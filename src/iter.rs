use util::*;
use super::*;

/// The iterator over the processed arguments.
///
/// # Parameters
///
/// `<'a>` – the lifetime of app’s [`Config`](struct.Config.html)
///
/// `<'b>` - the lifetime of the argument processing actions (closures) in the `Config`
///
/// `<I>`  – the underlying `String` iterator from which we are getting the unprocessed arguments
///
/// `<T>`  – the type into which each argument is parsed
#[derive(Debug)]
pub struct Iter<'a, 'b: 'a, I, T: 'a>
    where I: IntoIterator<Item=String>
{
    config:     &'a Config<'b, T>,
    args:       I::IntoIter,
    push_back:  Option<String>,
    positional: bool,
}

impl<'a, 'b, I, T> Iter<'a, 'b, I, T>
    where I: IntoIterator<Item=String>
{
    fn parse_positional(&self, actual: &str) -> Result<T> {
        let formal = self.config.get_positional()
            .ok_or_else(|| Error::from_string("Positional arguments not accepted"))?;
        formal.parse_argument(actual)
    }
}

impl<'a, 'b, I, T> Iterator for Iter<'a, 'b, I, T>
    where I: IntoIterator<Item=String>
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Result<T>> {
        use self::ArgState::*;

        let item = self.push_back.take().or_else(|| self.args.next())?;
        let arg  = item.as_str();

        if self.positional {
            return Some(self.parse_positional(arg));
        }

        match analyze_argument(arg) {
            EndOfOptions          => {
                self.positional = true;
                self.args.next().as_ref().map(|s| self.parse_positional(s))
            }

            ShortOption(c, param) => {
                let result = if let Some(arg) = self.config.get_short(c) {
                    if arg.takes_parameter() {
                        if !param.is_empty() {
                            arg.parse_argument(param)
                        } else if let Some(param) = self.args.next() {
                            arg.parse_argument(&param)
                        } else {
                            Err(arg.new_error(false, "expected option parameter"))
                        }
                    } else {
                        if !param.is_empty() {
                            self.push_back = Some(format!("-{}", param));
                        }
                        arg.parse_argument("")
                    }
                } else {
                    Err(Error::from_string("unrecognized").with_option(format!("-{}", c)))
                };

                Some(result)
            }

            LongOption(s, param)  => {
                let result = if let Some(arg) = self.config.get_long(s) {
                    if arg.takes_parameter() {
                        if let Some(param) = param {
                            arg.parse_argument(param)
                        } else if let Some(param) = self.args.next() {
                            arg.parse_argument(&param)
                        } else {
                            Err(arg.new_error(true, "expected option parameter"))
                        }
                    } else if param.is_none() {
                        arg.parse_argument("")
                    } else {
                        Err(arg.new_error(true, "unexpected option parameter"))
                    }
                } else {
                    Err(Error::from_string("unrecognized").with_option(format!("--{}", s)))
                };

                Some(result)
            }

            Positional(s)         => Some(self.parse_positional(s)),
        }.map(|o| o.map_err(|e| e.with_option(arg)))
    }
}

impl<'a, 'b, I, T> Iter<'a, 'b, I, T>
    where I: IntoIterator<Item=String>
{
    /// Creates a new `foropts::Iter` from a reference to the
    /// configuration and an iterator over the unparsed arguments.
    pub (crate) fn new(config: &'a Config<'b, T>, args: I) -> Self {
        Iter {
            config,
            args:       args.into_iter(),
            push_back:  None,
            positional: false,
        }
    }
}

enum ArgState<'a> {
    EndOfOptions,
    ShortOption(char, &'a str),
    LongOption(&'a str, Option<&'a str>),
    Positional(&'a str),
}

fn analyze_argument(param: &str) -> ArgState {
    match split_first_str(param) {
        Some(('-', rest)) => analyze_option(rest),
        _ => ArgState::Positional(param)
    }
}

fn analyze_option(opt: &str) -> ArgState {
    use self::ArgState::*;

    match split_first_str(opt) {
        None              => Positional("-"),
        Some(('-', ""))   => EndOfOptions,
        Some(('-', rest)) => {
            if let Some(ix) = rest.find('=') {
                LongOption(&rest[..ix], Some(&rest[ix + 1 ..]))
            } else {
                LongOption(rest, None)
            }
        }
        Some((c, rest))   => ShortOption(c, rest),
    }
}
