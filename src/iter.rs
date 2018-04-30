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
                return self.config.parse_positional(arg);
            }

            if arg == "--" {
                self.positional = true;
                return self.args.next().and_then(|arg| self.config.parse_positional(&arg));
            }

            if let Some(c) = arg.chars().next() {
                if c == '-' {
                    arg = &arg[1..];

                    for each in self.config.get_args() {
                        if let Some(result) = each.parse_optional(&mut arg, &mut self.args) {
                            if !arg.is_empty() {
                                self.push_back = Some(format!("-{}", arg));
                            }

                            return Some(result);
                        }
                    }

                    Some(Err(Error::from_string("unrecognized").with_option(arg)))
                } else {
                    self.config.parse_positional(arg)
                }
            } else {
                self.config.parse_positional("")
            }
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
