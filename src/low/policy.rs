#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Whether a particular option expects/recognizes parameters.
///
/// When queried about a flag, a [`Config`] returns a `Presence` to determine
/// how to parse it—in particular, whether to expect a parameter to follow.
///
/// Functions that accept `Presence`s usually accept any type that implements
/// `Into<Presence>`. This includes `bool`, where `true` maps to [`Always`]
/// and `false` maps to [`Never`].
///
/// [`Config`]: trait.Config.html
/// [`Always`]: #variant.Always
/// [`Never`]: #variant.Never
pub enum Presence {
    /// Option will expect a parameter.
    ///
    /// If a parameter is not found,
    /// [`ErrorKind::MissingParam`](enum.ErrorKind.html#variant.MissingParam)
    /// is returned. This can only happen at the end of the arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// # use foropts::low::*;
    /// let config = FnConfig::new(|_| Some(Presence::Always));
    ///
    /// let result: Vec<_> = config.slice_iter(&[
    ///     "-a", "foo", "-bbar", "--cee", "baz", "--dee=qux", "-e"
    /// ]).collect();
    ///
    /// assert_eq!( result,
    ///             &[Item::Opt(Flag::Short('a'), Some("foo"), ()),
    ///               Item::Opt(Flag::Short('b'), Some("bar"), ()),
    ///               Item::Opt(Flag::Long("cee"), Some("baz"), ()),
    ///               Item::Opt(Flag::Long("dee"), Some("qux"), ()),
    ///               Item::Error(ErrorKind::MissingParam(Flag::Short('e')))] );
    /// ```
    Always,
    /// Option will recognize a parameter if attached.
    ///
    /// For short options, this means that anything in the token following
    /// the flag will be considered the parameter; for long options, an
    /// equals sign (`=`) must be provided to recognize a parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use foropts::low::*;
    /// let config = FnConfig::new(|_| Some(Presence::IfAttached));
    ///
    /// let result: Vec<_> = config.slice_iter(&[
    ///     "-a", "foo", "-bbar", "--cee", "baz", "--dee=qux", "-e"
    /// ]).collect();
    ///
    /// assert_eq!( result,
    ///             &[Item::Opt(Flag::Short('a'), None, ()),
    ///               Item::Positional("foo"),
    ///               Item::Opt(Flag::Short('b'), Some("bar"), ()),
    ///               Item::Opt(Flag::Long("cee"), None, ()),
    ///               Item::Positional("baz"),
    ///               Item::Opt(Flag::Long("dee"), Some("qux"), ()),
    ///               Item::Opt(Flag::Short('e'), None, ()) ]);
    /// ```
    IfAttached,
    /// Option will not expect a parameter.
    ///
    /// It is an error to provide a long option with an equals sign (`=`), in
    /// which case
    /// [`ErrorKind::UnexpectedParam`](enum.ErrorKind.html#variant.UnexpectedParam)
    /// will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use foropts::low::*;
    /// let config = FnConfig::new(|_| Some(Presence::Never));
    ///
    /// let result: Vec<_> = config.slice_iter(&[
    ///     "-a", "foo", "-bbar", "--cee", "baz", "--dee=qux", "-e"
    /// ]).collect();
    ///
    /// assert_eq!( result,
    ///             &[Item::Opt(Flag::Short('a'), None, ()),
    ///               Item::Positional("foo"),
    ///               Item::Opt(Flag::Short('b'), None, ()),
    ///               Item::Opt(Flag::Short('b'), None, ()),
    ///               Item::Opt(Flag::Short('a'), None, ()),
    ///               Item::Opt(Flag::Short('r'), None, ()),
    ///               Item::Opt(Flag::Long("cee"), None, ()),
    ///               Item::Positional("baz"),
    ///               Item::Error(ErrorKind::UnexpectedParam(Flag::Long("dee"), "qux")),
    ///               Item::Opt(Flag::Short('e'), None, ()) ]);
    /// ```
    Never,
}

impl From<bool> for Presence {
    fn from(b: bool) -> Self {
        if b { Presence::Always } else { Presence::Never }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy<T> {
    pub presence: Presence,
    pub token:    T,
}

impl<T> Policy<T> {
    pub fn new<P>(presence: P, token: T) -> Self
        where P: Into<Presence> {

        Policy {
            presence: presence.into(),
            token,
        }
    }
}

impl<P> From<P> for Policy<()>
    where P: Into<Presence> {

    fn from(presence: P) -> Self {
        Policy {
            presence: presence.into(),
            token:    (),
        }
    }
}

impl<P, T> From<(P, T)> for Policy<T>
    where P: Into<Presence> {

    fn from((presence, token): (P, T)) -> Self {
        Policy {
            presence: presence.into(),
            token,
        }
    }
}
