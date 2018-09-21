use super::slice_iter::SliceIter;
use super::flag::Flag;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Presence {
    /// Option will expect a parameter.
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
    ///             &[Item::Opt(Flag::Short('a'), Some("foo")),
    ///               Item::Opt(Flag::Short('b'), Some("bar")),
    ///               Item::Opt(Flag::Long("cee"), Some("baz")),
    ///               Item::Opt(Flag::Long("dee"), Some("qux")),
    ///               Item::Error(ErrorKind::MissingParam(Flag::Short('e')))] );
    /// ```
    Always,
    /// Option will recognize a parameter if attached.
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
    ///             &[Item::Opt(Flag::Short('a'), None),
    ///               Item::Positional("foo"),
    ///               Item::Opt(Flag::Short('b'), Some("bar")),
    ///               Item::Opt(Flag::Long("cee"), None),
    ///               Item::Positional("baz"),
    ///               Item::Opt(Flag::Long("dee"), Some("qux")),
    ///               Item::Opt(Flag::Short('e'), None) ]);
    /// ```
    IfAttached,
    /// Option will not expect a parameter.
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
    ///             &[Item::Opt(Flag::Short('a'), None),
    ///               Item::Positional("foo"),
    ///               Item::Opt(Flag::Short('b'), None),
    ///               Item::Opt(Flag::Short('b'), None),
    ///               Item::Opt(Flag::Short('a'), None),
    ///               Item::Opt(Flag::Short('r'), None),
    ///               Item::Opt(Flag::Long("cee"), None),
    ///               Item::Positional("baz"),
    ///               Item::Error(ErrorKind::UnexpectedParam(Flag::Long("dee"), "qux")),
    ///               Item::Opt(Flag::Short('e'), None) ]);
    /// ```
    Never,
}

impl From<bool> for Presence {
    fn from(b: bool) -> Self {
        if b { Presence::Always } else { Presence::Never }
    }
}

pub trait Config {
    fn get_short_param(&self, short: char) -> Option<Presence>;

    fn get_long_param(&self, long: &str) -> Option<Presence>;

    fn slice_iter<'a, Arg>(&self, args: &'a [Arg]) -> SliceIter<'a, &Self, Arg>
        where Arg: Borrow<str> {
        
        SliceIter::new(self, args)
    }

    fn into_slice_iter<Arg>(self, args: &[Arg]) -> SliceIter<Self, Arg>
        where Self: Sized,
              Arg:  Borrow<str> {

        SliceIter::new(self, args)
    }
}

impl<'a, T: Config + ?Sized> Config for &'a T {
    fn get_short_param(&self, short: char) -> Option<Presence> {
        T::get_short_param(self, short)
    }

    fn get_long_param(&self, long: &str) -> Option<Presence> {
        T::get_long_param(self, long)
    }
}

impl<T: Config + ?Sized> Config for Box<T> {
    fn get_short_param(&self, short: char) -> Option<Presence> {
        T::get_short_param(&self, short)
    }

    fn get_long_param(&self, long: &str) -> Option<Presence> {
        T::get_long_param(&self, long)
    }
}

/// The configuration for the argument parser.
#[derive(Clone, Debug)]
pub struct HashConfig<L>
    where L: Eq + Hash + Borrow<str> {

    short_opts: HashMap<char, Presence>,
    long_opts:  HashMap<L, Presence>,
}

impl<L> Config for HashConfig<L>
    where L: Eq + Hash + Borrow<str> {

    fn get_short_param(&self, short: char) -> Option<Presence> {
        self.short_opts.get(&short).cloned()
    }

    fn get_long_param(&self, long: &str) -> Option<Presence> {
        self.long_opts.get(long).cloned()
    }
}

impl<L> HashConfig<L>
    where L: Eq + Hash + Borrow<str> {

    pub fn new() -> Self {
        HashConfig {
            short_opts: HashMap::new(),
            long_opts:  HashMap::new(),
        }
    }

    pub fn with_capacities(shorts: usize, longs: usize) -> Self {
        HashConfig {
            short_opts: HashMap::with_capacity(shorts),
            long_opts:  HashMap::with_capacity(longs),
        }
    }

    pub fn opt<F, P>(self, flag: F, param: P) -> Self
        where F: Into<Flag<L>>,
              P: Into<Presence> {

        match flag.into() {
            Flag::Short(short) => self.short(short, param),
            Flag::Long(long)   => self.long(long, param),
        }
    }

    pub fn short<P>(mut self, flag: char, param: P) -> Self
        where P: Into<Presence> {

        self.short_opts.insert(flag, param.into());
        self
    }

    pub fn long<S, P>(mut self, flag: S, param: P) -> Self
        where S: Into<L>,
              P: Into<Presence> {

        self.long_opts.insert(flag.into(), param.into());
        self
    }

    pub fn both<S, P>(self, short: char, long: S, param: P) -> Self
        where S: Into<L>,
              P: Into<Presence> {

        let presence = param.into();
        self.short(short, presence).long(long, presence)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FnConfig<F, P> {
    fun:     F,
    phantom: PhantomData<fn() -> P>,
}

impl<F, P> FnConfig<F, P> {
    pub fn new(fun: F) -> Self
        where F: Fn(Flag<&str>) -> Option<P>,
              P: Into<Presence> {

        FnConfig {
            fun,
            phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> F {
        self.fun
    }
}

impl<F, P> Deref for FnConfig<F, P> {
    type Target = F;

    fn deref(&self) -> &F {
        &self.fun
    }
}

impl<F, P> DerefMut for FnConfig<F, P> {
    fn deref_mut(&mut self) -> &mut F {
        &mut self.fun
    }
}

impl<F, P> Config for FnConfig<F, P>
    where F: Fn(Flag<&str>) -> Option<P>,
          P: Into<Presence> {
    
    fn get_short_param(&self, short: char) -> Option<Presence> {
        (self.fun)(Flag::Short(short)).map(Into::into)
    }

    fn get_long_param(&self, long: &str) -> Option<Presence> {
        (self.fun)(Flag::Long(long)).map(Into::into)
    }
}

impl<T: Config, U: Config> Config for (T, U) {
    fn get_short_param(&self, short: char) -> Option<Presence> {
        self.0.get_short_param(short).or_else(||
            self.1.get_short_param(short))
    }

    fn get_long_param(&self, long: &str) -> Option<Presence> {
        self.0.get_long_param(long).or_else(||
            self.1.get_long_param(long))
    }
}

impl<L, P> Config for [(Flag<L>, P)]
    where L: Borrow<str>,
          P: Copy + Into<Presence> {

    fn get_short_param(&self, name: char) -> Option<Presence> {
        self.into_iter().find(|pair| pair.0.is(name)).map(|pair| pair.1.into())
    }

    fn get_long_param(&self, name: &str) -> Option<Presence> {
        self.into_iter().find(|pair| pair.0.is(name)).map(|pair| pair.1.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::flag::Flag::*;
    use super::super::config::Presence::*;

    fn check_config<C: Config>(config: C) {
        assert_eq!( config.get_short_param('a'),        Some(Never) );
        assert_eq!( config.get_short_param('b'),        None );
        assert_eq!( config.get_short_param('m'),        Some(Always) );
        assert_eq!( config.get_short_param('r'),        Some(IfAttached) );
        assert_eq!( config.get_short_param('s'),        Some(Always) );
        assert_eq!( config.get_short_param('v'),        Some(Never) );
        assert_eq!( config.get_short_param('q'),        Some(Never) );

        assert_eq!( config.get_long_param("all"),       Some(Never) );
        assert_eq!( config.get_long_param("bare"),      None );
        assert_eq!( config.get_long_param("log"),       Some(IfAttached) );
        assert_eq!( config.get_long_param("message"),   Some(Always) );
        assert_eq!( config.get_long_param("rebase"),    Some(IfAttached) );
        assert_eq!( config.get_long_param("strategy"),  Some(Always) );
        assert_eq!( config.get_long_param("verbose"),   Some(Never) );
        assert_eq!( config.get_long_param("quiet"),     Some(Never) );
    }

    #[test]
    fn hash_config() {
        let config: HashConfig<String> = HashConfig::new()
            .both('a', "all",       false)
            .long(     "log",       IfAttached)
            .both('m', "message",   true)
            .both('r', "rebase",    IfAttached)
            .both('s', "strategy",  Always)
            .both('v', "verbose",   Never)
            .both('q', "quiet",     Never);

        check_config(config);
    }

    #[test]
    fn bare_bool_config() {
        let config = [
            (Short('a'), false),    (Long("all"), false),
            (Short('m'), true),     (Long("message"), true),
        ];

        assert_eq!( config.get_short_param('a'),        Some(Never) );
        assert_eq!( config.get_short_param('b'),        None );
        assert_eq!( config.get_short_param('m'),        Some(Always) );

        assert_eq!( config.get_long_param("all"),       Some(Never) );
        assert_eq!( config.get_long_param("bare"),      None );
        assert_eq!( config.get_long_param("message"),   Some(Always) );
    }

    #[test]
    fn bare_presence_config() {
        let config = [
            (Short('a'), Never),      (Long("all"),      Never),
                                      (Long("log"),      IfAttached),
            (Short('m'), Always),     (Long("message"),  Always),
            (Short('r'), IfAttached), (Long("rebase"),   IfAttached),
            (Short('s'), Always),     (Long("strategy"), Always),
            (Short('v'), Never),      (Long("verbose"),  Never),
            (Short('q'), Never),      (Long("quiet"),    Never),
        ];

        check_config(&config as &[_]);
    }

    #[test]
    fn pair_config() {
        let common_config = [
            (Short('v'), Never),      (Long("verbose"),  Never),
            (Short('q'), Never),      (Long("quiet"),    Never),
        ];

        let specific_config: HashConfig<&str> = HashConfig::new()
            .both('a', "all",       false)
            .long(     "log",       IfAttached)
            .both('m', "message",   true)
            .both('r', "rebase",    IfAttached)
            .both('s', "strategy",  Always);

        let config = (specific_config, &common_config as &[_]);

        check_config(config);
    }

    #[test]
    fn fn_config() {
        fn get(flag: Flag<&str>) -> Option<Presence> {
            let presence = match flag {
                Short('a')       => Never,
                Long("all")      => Never,
                Long("log")      => IfAttached,
                Short('m')       => Always,
                Long("message")  => Always,
                Short('r')       => IfAttached,
                Long("rebase")   => IfAttached,
                Short('s')       => Always,
                Long("strategy") => Always,
                Short('v')       => Never,
                Long("verbose")  => Never,
                Short('q')       => Never,
                Long("quiet")    => Never,
                _                => return None,
            };
            Some(presence)
        }

        let config = FnConfig::new(get);

        check_config(config);
        assert_eq!( config(Short('m')), Some(Always) );
    }

    #[test]
    fn allow_everything() {
        let config = FnConfig::new(|_| Some(IfAttached));
        assert_eq!( config.get_short_param('q'), Some(IfAttached) );
        assert_eq!( config.get_long_param("tralala"), Some(IfAttached) );
    }
}
