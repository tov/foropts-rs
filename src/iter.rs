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

impl<'a, 'b, I, T> Iterator for Iter<'a, 'b, I, T>
    where I: IntoIterator<Item=String>
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Result<T>> {
        self.push_back.take().or_else(|| self.args.next()).and_then(|item| {
            let mut arg = item.as_str();

            if self.positional {
                return Some(self.config.parse_positional(arg));
            }

            if arg == "--" {
                self.positional = true;
                return self.args.next().map(|arg| self.config.parse_positional(&arg));
            }

            let result = match arg.chars().next() {
                Some('-') => {
                    arg = &arg[1..];

                    for each in self.config.get_args() {
                        if let Some((result, rest)) = each.parse_optional(arg, &mut self.args) {
                            if !rest.is_empty() {
                                self.push_back = Some(format!("-{}", rest));
                            }

                            return Some(result);
                        }
                    }

                    Err(Error::from_string("unrecognized").with_option(arg))
                }

                Some(_)   => self.config.parse_positional(arg),

                None      => self.config.parse_positional(""),
            };

            Some(result)
        })
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

enum ParamState<'a> {
    EndOfOptions,
    ShortOption(char, &'a str),
    LongOption(&'a str, Option<&'a str>),
    Positional(&'a str),
}

fn analyze_parameter(param: &str) -> ParamState {
    match split_first_str(param) {
        Some(('-', rest)) => analyze_option(rest),
        _ => ParamState::Positional(param)
    }
}

fn analyze_option(opt: &str) -> ParamState {
    use self::ParamState::*;

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
