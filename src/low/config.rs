use super::slice_iter::SliceIter;
use super::flag::Flag;
use super::policy::{Policy, Presence};

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub trait Config {
    type Token;

    fn get_short_policy(&self, short: char) -> Option<Policy<Self::Token>>;

    fn get_long_policy(&self, long: &str) -> Option<Policy<Self::Token>>;

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

impl<'a, C: Config + ?Sized> Config for &'a C {
    type Token = C::Token;

    fn get_short_policy(&self, short: char) -> Option<Policy<Self::Token>> {
        C::get_short_policy(self, short)
    }

    fn get_long_policy(&self, long: &str) -> Option<Policy<Self::Token>> {
        C::get_long_policy(self, long)
    }
}

impl<C: Config + ?Sized> Config for Box<C> {
    type Token = C::Token;

    fn get_short_policy(&self, short: char) -> Option<Policy<Self::Token>> {
        C::get_short_policy(&self, short)
    }

    fn get_long_policy(&self, long: &str) -> Option<Policy<Self::Token>> {
        C::get_long_policy(&self, long)
    }
}

pub type HashConfig<L> = TokenHashConfig<L, ()>;

/// The configuration for the argument parser.
#[derive(Clone)]
pub struct TokenHashConfig<L, T> {
    short_opts: HashMap<char, Policy<T>>,
    long_opts:  HashMap<L, Policy<T>>,
}

impl<L, T> fmt::Debug for TokenHashConfig<L, T>
    where L: Eq + Hash + Borrow<str>,
          T: fmt::Debug {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut map = f.debug_map();

        for (key, value) in &self.short_opts {
            map.entry(&Flag::Short::<()>(*key), value);
        }

        for (key, value) in &self.long_opts {
            map.entry(&Flag::Long(key.borrow()), value);
        }

        map.finish()
    }
}

impl<L, T> Config for TokenHashConfig<L, T>
    where L: Eq + Hash + Borrow<str>,
          T: Clone {

    type Token = T;

    fn get_short_policy(&self, short: char) -> Option<Policy<T>> {
        self.short_opts.get(&short).cloned()
    }

    fn get_long_policy(&self, long: &str) -> Option<Policy<T>> {
        self.long_opts.get(long).cloned()
    }
}

impl<L, T> TokenHashConfig<L, T>
    where L: Eq + Hash + Borrow<str> {

    pub fn new() -> Self {
        TokenHashConfig {
            short_opts: HashMap::new(),
            long_opts:  HashMap::new(),
        }
    }

    pub fn with_capacities(shorts: usize, longs: usize) -> Self {
        TokenHashConfig {
            short_opts: HashMap::with_capacity(shorts),
            long_opts:  HashMap::with_capacity(longs),
        }
    }

    pub fn opt<F, P>(self, flag: F, param: P) -> Self
        where F: Into<Flag<L>>,
              P: Into<Policy<T>> {

        match flag.into() {
            Flag::Short(short) => self.short(short, param),
            Flag::Long(long)   => self.long(long, param),
        }
    }

    pub fn short<P>(mut self, flag: char, param: P) -> Self
        where P: Into<Policy<T>> {

        self.short_opts.insert(flag, param.into());
        self
    }

    pub fn long<S, P>(mut self, flag: S, param: P) -> Self
        where S: Into<L>,
              P: Into<Policy<T>> {

        self.long_opts.insert(flag.into(), param.into());
        self
    }

    pub fn both<S, P>(self, short: char, long: S, param: P) -> Self
        where S: Into<L>,
              P: Clone + Into<Policy<T>> {

        self.short(short, param.clone().into()).long(long, param.into())
    }
}

pub type FnConfig<F, P> = TokenFnConfig<F, P, ()>;

#[derive(Debug, Clone, Copy)]
pub struct TokenFnConfig<F, P, T> {
    fun:     F,
    phantom: PhantomData<fn() -> (P, T)>,
}

impl<F, P, T> TokenFnConfig<F, P, T> {
    pub fn new(fun: F) -> Self
        where F: Fn(Flag<&str>) -> Option<P>,
              P: Into<Policy<T>> {

        TokenFnConfig {
            fun,
            phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> F {
        self.fun
    }
}

impl<F, P, T> Deref for TokenFnConfig<F, P, T> {
    type Target = F;

    fn deref(&self) -> &F {
        &self.fun
    }
}

impl<F, P, T> DerefMut for TokenFnConfig<F, P, T> {
    fn deref_mut(&mut self) -> &mut F {
        &mut self.fun
    }
}

impl<F, P, T> Config for TokenFnConfig<F, P, T>
    where F: Fn(Flag<&str>) -> Option<P>,
          P: Into<Policy<T>> {

    type Token = T;

    fn get_short_policy(&self, short: char) -> Option<Policy<T>> {
        (self.fun)(Flag::Short(short)).map(Into::into)
    }

    fn get_long_policy(&self, long: &str) -> Option<Policy<T>> {
        (self.fun)(Flag::Long(long)).map(Into::into)
    }
}

impl<C1, C2> Config for (C1, C2)
    where C1: Config,
          C2: Config<Token = C1::Token> {

    type Token = C1::Token;

    fn get_short_policy(&self, short: char) -> Option<Policy<Self::Token>> {
        self.0.get_short_policy(short).or_else(||
            self.1.get_short_policy(short))
    }

    fn get_long_policy(&self, long: &str) -> Option<Policy<Self::Token>> {
        self.0.get_long_policy(long).or_else(||
            self.1.get_long_policy(long))
    }
}

impl<L, P> Config for [(Flag<L>, P)]
    where L: Borrow<str>,
          P: Clone + Into<Presence> {

    type Token = ();

    fn get_short_policy(&self, name: char) -> Option<Policy<()>> {
        self.into_iter().find(|pair| pair.0.is(name))
            .map(|pair| Policy::new(pair.1.clone(), ()))
    }

    fn get_long_policy(&self, name: &str) -> Option<Policy<()>> {
        self.into_iter().find(|pair| pair.0.is(name))
            .map(|pair| Policy::new(pair.1.clone(), ()))
    }
}

impl<L, P, T> Config for [(Flag<L>, P, T)]
    where L: Borrow<str>,
          P: Clone + Into<Presence>,
          T: Clone {

    type Token = T;

    fn get_short_policy(&self, name: char) -> Option<Policy<T>> {
        self.into_iter().find(|pair| pair.0.is(name))
            .map(|&(_, ref presence, ref token)|
                Policy::new(presence.clone(), token.clone()))
    }

    fn get_long_policy(&self, name: &str) -> Option<Policy<T>> {
        self.into_iter().find(|pair| pair.0.is(name))
            .map(|&(_, ref presence, ref token)|
                Policy::new(presence.clone(), token.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Flag::*;
    use super::super::Presence::*;

    fn get_short<C: Config>(config: C, name: char) -> Option<Presence> {
        config.get_short_policy(name).map(|policy| policy.presence)
    }

    fn get_long<C: Config>(config: C, name: &str) -> Option<Presence> {
        config.get_long_policy(name).map(|policy| policy.presence)
    }

    fn check_config<C: Config>(config: C) {
        assert_eq!(get_short(&config, 'a'), Some(Never) );
        assert_eq!(get_short(&config, 'b'), None );
        assert_eq!(get_short(&config, 'm'), Some(Always) );
        assert_eq!(get_short(&config, 'r'), Some(IfAttached) );
        assert_eq!(get_short(&config, 's'), Some(Always) );
        assert_eq!(get_short(&config, 'v'), Some(Never) );
        assert_eq!(get_short(&config, 'q'), Some(Never) );

        assert_eq!(get_long(&config, "all"), Some(Never) );
        assert_eq!(get_long(&config, "bare"), None );
        assert_eq!(get_long(&config, "log"), Some(IfAttached) );
        assert_eq!(get_long(&config, "message"), Some(Always) );
        assert_eq!(get_long(&config, "rebase"), Some(IfAttached) );
        assert_eq!(get_long(&config, "strategy"), Some(Always) );
        assert_eq!(get_long(&config, "verbose"), Some(Never) );
        assert_eq!(get_long(&config, "quiet"), Some(Never) );
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
        let config: &[_] = &[
            (Short('a'), false),    (Long("all"), false),
            (Short('m'), true),     (Long("message"), true),
        ];

        assert_eq!( get_short(&config, 'a'),        Some(Never) );
        assert_eq!( get_short(&config, 'b'),        None );
        assert_eq!( get_short(&config, 'm'),        Some(Always) );

        assert_eq!( get_long(&config, "bare"),      None );
        assert_eq!( get_long(&config, "all"),       Some(Never) );
        assert_eq!( get_long(&config, "message"),   Some(Always) );
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
        assert_eq!( get_short(&config, 'q'), Some(IfAttached) );
        assert_eq!( get_long(&config, "tralala"), Some(IfAttached) );
    }
}
