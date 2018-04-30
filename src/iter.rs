use super::*;

#[derive(Debug)]
pub struct Iter<'a, 'b: 'a, I, T: 'a>
    where I: IntoIterator<Item=String>
{
    pub (crate) config:     &'a Config<'b, T>,
    pub (crate) args:       I::IntoIter,
    pub (crate) push_back:  Option<String>,
    pub (crate) positional: bool,
}

impl<'a, 'b, I, T> Iterator for Iter<'a, 'b, I, T>
    where I: IntoIterator<Item=String>
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Result<T>> {
        self.push_back.take().or_else(|| self.args.next()).and_then(|item| {
            let mut arg = item.as_str();

            if let Some(c) = arg.chars().next() {
                if c == '-' {
                    arg = &arg[1..];

                    for each in &self.config.args {
                        if let Some(result) = each.parse_optional(&mut arg, &mut self.args) {
                            if !arg.is_empty() {
                                self.push_back = Some(format!("-{}", arg));
                            }

                            return Some(result);
                        }
                    }

                    let msg = format!("Unknown option: -{}", arg);
                    Some(Err(Error::from_string(msg)))
                } else {
                    self.config.parse_positional(arg)
                }
            } else {
                self.config.parse_positional("")
            }
        })
    }
}

