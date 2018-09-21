use std::fmt;

#[derive(Clone)]
pub (crate) enum InnerState<'a> {
    Start,
    ShortOpts(&'a str),
    PositionalOnly,
}

impl<'a> InnerState<'a> {
    pub (crate) fn fmt_to_debug_list(&self, list: &mut fmt::DebugList) {
        match *self {
            InnerState::Start => (),
            InnerState::ShortOpts(shorts) => {
                list.entry(&format!("-{}", shorts));
            }
            InnerState::PositionalOnly => {
                list.entry(&"--");
            }
        }
    }
}
